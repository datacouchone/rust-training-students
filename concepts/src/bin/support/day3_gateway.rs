#![allow(dead_code)]
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
pub struct Budget { pub used: AtomicUsize, pub limit: usize }
impl Budget {
    pub fn new(limit: usize) -> Arc<Self> {
        Arc::new(Self { used: AtomicUsize::new(0), limit })
    }
    pub fn used(&self) -> usize { self.used.load(Ordering::Relaxed) }
}
pub struct Debit { pub budget: Arc<Budget>, pub amount: usize }
impl Drop for Debit {
    fn drop(&mut self) { self.budget.used.fetch_sub(self.amount, Ordering::Relaxed); }
}
pub fn claim(budget: &Arc<Budget>, amount: usize) -> Option<Debit> {
    let mut old = budget.used.load(Ordering::Relaxed);
    loop {
        let next = old.checked_add(amount)?;
        if next > budget.limit { return None; }
        match budget.used.compare_exchange_weak(old, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Some(Debit { budget: Arc::clone(budget), amount }),
            Err(observed) => old = observed,
        }
    }
}
pub fn reserve_pair(global: &Arc<Budget>, tenant: &Arc<Budget>, amount: usize) -> Option<(Debit, Debit)> {
    let g = claim(global, amount)?;
    let t = claim(tenant, amount)?;
    Some((g, t))
}
pub fn require_send_sync<T: Send + Sync>() {}
