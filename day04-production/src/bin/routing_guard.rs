#[allow(unused_imports)]
use day04_production::{routing::{self, Locked, Published, Snapshot}, encoding::{self, Encoder, Policy}, analytics};
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};
fn main() {
    // TASK_BEGIN demonstrate
    let host = "tenant-17.example.net";
    let rows = routing::fixture(1, 4096);
    let snapshot = Snapshot::compile(1, rows).unwrap();
    let service = Published::new(snapshot);
    let request = service.0.load_full();
    let destination = request.route(host).unwrap();
    assert_eq!(destination.pool, 17);
    println!("generation={} pool={} timeout_ms={}", request.generation, destination.pool, destination.timeout_ms);
    // TASK_END demonstrate
}
