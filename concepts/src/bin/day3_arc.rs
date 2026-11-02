// Independent gateway concept demonstration. Main is the only replacement region.
fn main() {
    // TASK_BEGIN demonstrate
    let gateway = Budget::new(10);
    let accidental_copy = Budget::new(10);
    let _a = claim(&gateway, 7).unwrap();
    let _b = claim(&accidental_copy, 7).unwrap();
    println!("same budget={}; total promised={}", Arc::ptr_eq(&gateway, &accidental_copy), gateway.used() + accidental_copy.used());
    // TASK_END demonstrate
}
#[path = "support/day3_gateway.rs"]
mod gateway;
use gateway::*;
use std::sync::{Arc, Barrier, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
