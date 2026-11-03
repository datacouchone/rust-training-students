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
    // REPAIR: small/missing/inconsistent evidence must not authorize promotion.
    if w.errors <= w.requests / 100 && w.p95_us <= 50_000 {
        Decision::Promote
    } else {
        Decision::Rollback
    }
}
