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
        let mut old = self.used.load(Ordering::Relaxed);
        loop {
            let next = old.checked_add(bytes)?;
            if next > self.limit {
                return None;
            }
            // Numeric reservation only: this CAS does not publish/access unrelated memory.
            match self
                .used
                .compare_exchange_weak(old, next, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => {
                    return Some(Debit {
                        budget: self.clone(),
                        bytes,
                    });
                }
                Err(observed) => old = observed,
            }
        }
        // TASK_END debit
    }
}
impl Drop for Debit {
    fn drop(&mut self) {
        // TASK_BEGIN release
        // Each successful debit has exactly one non-Clone owner, including rollback paths.
        self.budget.used.fetch_sub(self.bytes, Ordering::Relaxed);
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
        if Arc::ptr_eq(global, tenant) {
            return Err(Rejected::SameBudget);
        }
        let g = global.debit(bytes).ok_or(Rejected::Global)?;
        let t = tenant.debit(bytes).ok_or(Rejected::Tenant)?;
        Ok(Self {
            _global: g,
            _tenant: t,
        })
        // TASK_END reserve
    }
}
