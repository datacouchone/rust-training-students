#[allow(unused_imports)]
use day04_production::{routing::{self, Locked, Published, Snapshot}, encoding::{self, Encoder, Policy}, analytics};
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};
fn main() {
    // TASK_BEGIN demonstrate
    let mut encoder = Encoder::new(64 * 1024, 1024).unwrap();
    let events = encoding::fixture(100, 16);
    let result = encoder.encode(&events, Policy::Bounded).map(|bytes| bytes.len());
    assert!(result.is_ok());
    println!("export={result:?}");
    assert!(encoder.encode(&events[..1], Policy::Bounded).is_ok());
    println!("next small batch succeeds");
    // TASK_END demonstrate
}
