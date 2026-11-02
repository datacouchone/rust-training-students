// Independent gateway concept demonstration. Main is the only replacement region.
fn main() {
    // TASK_BEGIN demonstrate
    let used = AtomicUsize::new(0);
    let (a_old, b_old) = (used.load(Ordering::SeqCst), used.load(Ordering::SeqCst));
    assert!(a_old + 7 <= 10 && b_old + 7 <= 10);
    used.store(a_old + 7, Ordering::SeqCst);
    used.store(b_old + 7, Ordering::SeqCst);
    println!("counter={}, admitted=14, limit=10", used.load(Ordering::SeqCst));
    // TASK_END demonstrate
}
#[path = "support/day3_gateway.rs"]
mod gateway;
use gateway::*;
use std::sync::{Arc, Barrier, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
