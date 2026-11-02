// Independent gateway concept demonstration. Main is the only replacement region.
fn main() {
    // TASK_BEGIN demonstrate
    let global = Budget::new(100);
    let tenant = Budget::new(4);
    global.used.store(5, Ordering::Relaxed);
    let accepted = claim(&tenant, 5).is_some();
    println!("accepted={accepted}; global={}, tenant={}", global.used(), tenant.used());
    // TASK_END demonstrate
}
#[path = "support/day3_gateway.rs"]
mod gateway;
use gateway::*;
use std::sync::{Arc, Barrier, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
