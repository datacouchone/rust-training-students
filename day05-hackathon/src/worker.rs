use crate::{
    boundary::Validated,
    ledger::{Ledger, Outcome, Reject},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::{
    sync::{Mutex, Semaphore},
    time::{Duration, Instant, timeout_at},
};
pub struct Service {
    pub ledger: Mutex<Ledger>,
    pub slots: Arc<Semaphore>,
    pub active: Arc<AtomicUsize>,
    pub peak: Arc<AtomicUsize>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Timeout,
    Closed,
    Rejected(Reject),
}
struct Active {
    active: Arc<AtomicUsize>,
}
impl Drop for Active {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}
impl Service {
    pub fn new(slots: usize, tenants: usize, receipts: usize) -> Arc<Self> {
        Arc::new(Self {
            ledger: Mutex::new(Ledger::new(tenants, receipts)),
            slots: Arc::new(Semaphore::new(slots)),
            active: Arc::new(AtomicUsize::new(0)),
            peak: Arc::new(AtomicUsize::new(0)),
        })
    }
    async fn operation(self: Arc<Self>, e: Validated, delay: Duration) -> Result<Outcome, Error> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(active, Ordering::SeqCst);
        let _active = Active {
            active: Arc::clone(&self.active),
        };
        // REPAIR: ledger access must not own the duration of the enrichment wait.
        let mut ledger = self.ledger.lock().await;
        tokio::time::sleep(delay).await;
        ledger.apply(&e).map_err(Error::Rejected)
    }
    pub async fn submit(
        self: &Arc<Self>,
        e: Validated,
        deadline: Instant,
        delay: Duration,
    ) -> Result<Outcome, Error> {
        // REPAIR: waiting for admission belongs to the same deadline.
        let permit = Arc::clone(&self.slots)
            .acquire_owned()
            .await
            .map_err(|_| Error::Closed)?;
        let child = tokio::spawn(Arc::clone(self).operation(e, delay));
        let result = timeout_at(deadline, child)
            .await
            .map_err(|_| Error::Timeout)?;
        drop(permit);
        result.map_err(|_| Error::Closed)?
    }
}
