// One focused Tokio experiment. Replace changes only the marked body.
// The runtime is single-threaded with a paused test clock; this is not a benchmark.
#[path = "support/day3_async.rs"]
mod support;
#[allow(unused_imports)]
use support::*;
#[allow(unused_imports)]
use std::{collections::BTreeMap, sync::{Arc, atomic::AtomicUsize}, time::Duration};
#[allow(unused_imports)]
use tokio::{sync::{oneshot, mpsc}, task::JoinSet, time};
fn main() {
    runtime().block_on(async {
        // TASK_BEGIN demonstrate
        let active = Arc::new(AtomicUsize::new(0));
        let n = Arc::clone(&active);
        let request = async move {
        let _slot = Slot::new(&n);
        std::future::pending::<()>().await;
        };
        assert_eq!(count(&active), 0);
        println!("created but unpolled: active={}", count(&active));
        drop(request);
        assert_eq!(count(&active), 0);
        // TASK_END demonstrate
    });
}
