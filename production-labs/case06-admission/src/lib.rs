//! Weighted admission across a process budget and a tenant budget.
//! Reservations account bytes only; they do not publish payload memory.
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
#[derive(Debug)]
pub struct Budget {
    limit: usize,
    used: AtomicUsize,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Rejected {
    Global,
    Tenant,
    SameBudget,
}
struct Debit {
    budget: Arc<Budget>,
    bytes: usize,
}
pub struct Lease {
    _global: Debit,
    _tenant: Debit,
}
impl Budget {
    pub fn new(limit: usize) -> Arc<Self> {
        Arc::new(Self {
            limit,
            used: AtomicUsize::new(0),
        })
    }
    pub fn used(&self) -> usize {
        self.used.load(Ordering::Relaxed)
    }
    pub fn limit(&self) -> usize {
        self.limit
    }
    fn debit(self: &Arc<Self>, bytes: usize) -> Option<Debit> {
        // TASK_BEGIN debit
        todo!("Implement debit: see README contract and acceptance tests");
        // TASK_END debit
    }
}
impl Drop for Debit {
    fn drop(&mut self) {
        // TASK_BEGIN release
        todo!("Implement release: see README contract and acceptance tests");
        // TASK_END release
    }
}
impl Lease {
    pub fn reserve(
        global: &Arc<Budget>,
        tenant: &Arc<Budget>,
        bytes: usize,
    ) -> Result<Self, Rejected> {
        // TASK_BEGIN reserve
        todo!("Implement reserve: see README contract and acceptance tests");
        // TASK_END reserve
    }
}
