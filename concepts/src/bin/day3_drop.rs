// Independent gateway concept demonstration. Main is the only replacement region.
fn main() {
    // TASK_BEGIN demonstrate
    let budget = Budget::new(10);
    let request = claim(&budget, 7).unwrap();
    println!("while request lives: {}", budget.used());
    assert_eq!(request.amount, 7);
    // TASK_END demonstrate
}
#[path = "support/day3_gateway.rs"]
mod gateway;
use gateway::*;
use std::sync::{Arc, Barrier, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
