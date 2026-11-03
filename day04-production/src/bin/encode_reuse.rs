#[allow(unused_imports)]
use day04_production::{routing::{self, Locked, Published, Snapshot}, encoding::{self, Encoder, Policy}, analytics};
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};
fn main() {
    // TASK_BEGIN demonstrate
    let policy = Policy::Reuse;
    let mut encoder = Encoder::new(4 * 1024 * 1024, 16 * 1024).unwrap();
    let small = encoding::fixture(32, 8);
    let burst = encoding::fixture(16_384, 4096);
    for (phase, events, count) in [("steady", &small, 100), ("burst", &burst, 1), ("quiet", &small, 100)] {
        let mut total = 0;
        for _ in 0..count {
            total += encoder.encode(events, policy).unwrap().len();
            encoder.reset(policy);
        }
        println!("{policy:?} {phase} bytes={total} retained_capacity={}", encoder.retained_capacity());
    }
    // TASK_END demonstrate
}
