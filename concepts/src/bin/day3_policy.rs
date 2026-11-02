// Independent gateway concept demonstration. Main is the only replacement region.
fn main() {
    // TASK_BEGIN demonstrate
    let (normal, critical) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let (a_total, b_total) = (normal.load(Ordering::SeqCst) + critical.load(Ordering::SeqCst), normal.load(Ordering::SeqCst) + critical.load(Ordering::SeqCst));
    assert!(a_total + 7 <= 10 && b_total + 7 <= 10);
    normal.store(7, Ordering::SeqCst);
    critical.store(7, Ordering::SeqCst);
    println!("separate fields: total={}", normal.load(Ordering::SeqCst) + critical.load(Ordering::SeqCst));
    // TASK_END demonstrate
}
#[path = "support/day3_gateway.rs"]
mod gateway;
use gateway::*;
use std::sync::{Arc, Barrier, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
