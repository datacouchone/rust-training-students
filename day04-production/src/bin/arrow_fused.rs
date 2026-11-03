#[allow(unused_imports)]
use day04_production::{routing::{self, Locked, Published, Snapshot}, encoding::{self, Encoder, Policy}, analytics};
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};
fn main() {
    // TASK_BEGIN demonstrate
    let batch = analytics::fixture(65_539, 17, 19);
    let threshold = 50_000;
    let expected = analytics::kernels(&batch, threshold).unwrap();
    let result = analytics::kernels(&batch, threshold).unwrap();
    assert_eq!(result, expected);
    println!("rows={} result={result:?}", batch.num_rows());
    // TASK_END demonstrate
}
