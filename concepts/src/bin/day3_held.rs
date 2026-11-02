// Independent gateway concept demonstration. Main is the only replacement region.
fn main() {
    // TASK_BEGIN demonstrate
    let budget = Budget::new(5);
    for _ in 0..16 { drop(claim(&budget, 1).unwrap()); }
    println!("end-only usage={}", budget.used());
    // TASK_END demonstrate
}
#[path = "support/day3_gateway.rs"]
mod gateway;
use gateway::*;
use std::sync::{Arc, Barrier, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
