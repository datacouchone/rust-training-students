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
    todo!("Implement attempt: see README contract and acceptance tests");
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
    todo!("Implement wait: see README contract and acceptance tests");
        // TASK_END wait
    }
    pub fn reconfigure(&self, config: Config) -> Result<(), Error> {
        // TASK_BEGIN reconfigure
    todo!("Implement reconfigure: see README contract and acceptance tests");
        // TASK_END reconfigure
    }
    pub fn close(&self) {
        // TASK_BEGIN close
    todo!("Implement close: see README contract and acceptance tests");
        // TASK_END close
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        // TASK_BEGIN release
    todo!("Implement release: see README contract and acceptance tests");
        // TASK_END release
    }
}
