// Independent gateway concept demonstration. Main is the only replacement region.
fn main() {
    // TASK_BEGIN demonstrate
    let budget = Budget::new(10);
    let _lease = claim(&budget, 7).unwrap();
    let payload: Option<Vec<u8>> = None;
    println!("reserved={}; payload_ready={}", budget.used(), payload.is_some());
    // TASK_END demonstrate
}
#[path = "support/day3_gateway.rs"]
mod gateway;
use gateway::*;
use std::sync::{Arc, Barrier, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
