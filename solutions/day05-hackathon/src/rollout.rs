#[derive(Clone, Copy)]
pub struct Window {
    pub requests: u64,
    pub successes: u64,
    pub errors: u64,
    pub p95_us: u64,
    pub duplicates: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Hold,
    Promote,
    Rollback,
}
/// Exercise policy: >=1000 observations, <=1% errors, p95<=50ms, no duplicate effects.
/// These are classroom service thresholds, not universal SLOs or statistical confidence.
pub fn decide(w: Window) -> Decision {
    if w.requests == 0 || w.successes.checked_add(w.errors) != Some(w.requests) {
        return Decision::Hold;
    }
    if w.duplicates > 0 {
        return Decision::Rollback;
    }
    if w.requests < 1000 {
        return Decision::Hold;
    }
    if u128::from(w.errors) * 100 > u128::from(w.requests) || w.p95_us > 50_000 {
        Decision::Rollback
    } else {
        Decision::Promote
    }
}
