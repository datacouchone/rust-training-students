//! Original admission-control challenge: reserved critical capacity, live policy changes and close.
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Normal,
    Critical,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub capacity: usize,
    pub critical_reserve: usize,
}
impl Config {
    pub fn validate(self) -> Result<(), Error> {
        if self.capacity == 0 || self.critical_reserve > self.capacity {
            Err(Error::InvalidConfig)
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidConfig,
    ZeroWeight,
    TooLarge,
    Busy,
    Timeout,
    Closed,
    WouldOvercommit,
    InvalidDeadline,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    pub config: Config,
    pub normal: usize,
    pub critical: usize,
    pub closed: bool,
}
#[derive(Debug)]
pub struct Pool {
    state: Mutex<Snapshot>,
    changed: Condvar,
}
#[derive(Debug)]
pub struct Lease {
    pool: Arc<Pool>,
    class: Class,
    weight: usize,
}
impl Pool {
    pub fn new(config: Config) -> Result<Arc<Self>, Error> {
        config.validate()?;
        Ok(Arc::new(Self {
            state: Mutex::new(Snapshot {
                config,
                normal: 0,
                critical: 0,
                closed: false,
            }),
            changed: Condvar::new(),
        }))
    }
    pub fn snapshot(&self) -> Snapshot {
        *self.state.lock().unwrap()
    }
    // Called only while the single state lock is held. Rejection must not mutate it.
    fn attempt(s: &mut Snapshot, class: Class, weight: usize) -> Result<(), Error> {
        // TASK_BEGIN attempt
        if s.closed {
            return Err(Error::Closed);
        }
        if weight == 0 {
            return Err(Error::ZeroWeight);
        }
        let ceiling = if class == Class::Normal {
            s.config.capacity - s.config.critical_reserve
        } else {
            s.config.capacity
        };
        if weight > ceiling {
            return Err(Error::TooLarge);
        }
        let total = s
            .normal
            .checked_add(s.critical)
            .and_then(|v| v.checked_add(weight))
            .ok_or(Error::Busy)?;
        if total > s.config.capacity {
            return Err(Error::Busy);
        }
        if class == Class::Normal {
            let next = s.normal.checked_add(weight).ok_or(Error::Busy)?;
            if next > ceiling {
                return Err(Error::Busy);
            }
            s.normal = next;
        } else {
            s.critical = s.critical.checked_add(weight).ok_or(Error::Busy)?;
        }
        Ok(())
        // TASK_END attempt
    }
    pub fn try_acquire(self: &Arc<Self>, class: Class, weight: usize) -> Result<Lease, Error> {
        Self::attempt(&mut self.state.lock().unwrap(), class, weight)?;
        Ok(Lease {
            pool: self.clone(),
            class,
            weight,
        })
    }
    /// OS-thread blocking API, deliberately not an async function.
    pub fn wait_acquire(
        self: &Arc<Self>,
        class: Class,
        weight: usize,
        timeout: Duration,
    ) -> Result<Lease, Error> {
        // TASK_BEGIN wait
        let until = Instant::now()
            .checked_add(timeout)
            .ok_or(Error::InvalidDeadline)?;
        let mut state = self.state.lock().unwrap();
        loop {
            match Self::attempt(&mut state, class, weight) {
                Ok(()) => {
                    return Ok(Lease {
                        pool: self.clone(),
                        class,
                        weight,
                    });
                }
                Err(Error::Busy) => {}
                Err(e) => return Err(e),
            }
            let now = Instant::now();
            if now >= until {
                return Err(Error::Timeout);
            }
            let (next, _) = self.changed.wait_timeout(state, until - now).unwrap();
            state = next;
            // A wakeup is a hint: all predicates must be rechecked under the lock.
        }
        // TASK_END wait
    }
    pub fn reconfigure(&self, config: Config) -> Result<(), Error> {
        // TASK_BEGIN reconfigure
        config.validate()?;
        let mut s = self.state.lock().unwrap();
        if s.closed {
            return Err(Error::Closed);
        }
        let used = s
            .normal
            .checked_add(s.critical)
            .ok_or(Error::WouldOvercommit)?;
        if used > config.capacity || s.normal > config.capacity - config.critical_reserve {
            return Err(Error::WouldOvercommit);
        }
        s.config = config;
        self.changed.notify_all();
        Ok(())
        // TASK_END reconfigure
    }
    pub fn close(&self) {
        // TASK_BEGIN close
        let mut s = self.state.lock().unwrap();
        s.closed = true;
        self.changed.notify_all();
        // TASK_END close
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        // TASK_BEGIN release
        let mut s = self.pool.state.lock().unwrap();
        match self.class {
            Class::Normal => s.normal -= self.weight,
            Class::Critical => s.critical -= self.weight,
        }
        self.pool.changed.notify_all();
        // TASK_END release
    }
}
