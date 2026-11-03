#[allow(unused_imports)]
use day04_production::{routing::{self, Locked, Published, Snapshot}, encoding::{self, Encoder, Policy}, analytics};
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};
fn main() {
    // TASK_BEGIN demonstrate
    let service = Published::new(Snapshot::compile(1, routing::fixture(1, 64)).unwrap());
    let request = service.0.load_full();
    service.reload(2, routing::fixture(2, 64)).unwrap();
    assert_eq!(request.generation, 1);
    println!("request={} published={} timeout_ms={}", request.generation, service.0.load().generation, request.route("tenant-17.example.net").unwrap().timeout_ms);
    let mut invalid = routing::fixture(3, 64);
    invalid.push(invalid[0].clone());
    assert!(service.reload(3, invalid).is_err());
    assert_eq!(service.0.load().generation, 2);
    // TASK_END demonstrate
}
