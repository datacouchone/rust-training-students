// Independent gateway concept demonstration. Main is the only replacement region.
fn main() {
    // TASK_BEGIN demonstrate
    let used = AtomicUsize::new(0);
    let b_old = used.load(Ordering::Relaxed);
    used.compare_exchange(0, 7, Ordering::Relaxed, Ordering::Relaxed).unwrap();
    let attempt = used.compare_exchange(b_old, b_old + 3, Ordering::Relaxed, Ordering::Relaxed);
    assert_eq!(attempt, Err(7));
    println!("stale attempt={attempt:?}; counter={}", used.load(Ordering::Relaxed));
    // TASK_END demonstrate
}
#[path = "support/day3_gateway.rs"]
mod gateway;
use gateway::*;
use std::sync::{Arc, Barrier, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
