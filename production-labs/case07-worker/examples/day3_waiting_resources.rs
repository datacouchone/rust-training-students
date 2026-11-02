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
        let (ready_tx, ready_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel::<()>();
        let n = Arc::clone(&active);
        let worker = tokio::spawn(async move {
        let slot = Slot::new(&n);
        drop(slot);
        ready_tx.send(()).unwrap();
        release_rx.await.unwrap();
        });
        ready_rx.await.unwrap();
        println!("waiting request: active={}", count(&active));
        assert_eq!(count(&active), 0);
        release_tx.send(()).unwrap();
        worker.await.unwrap();
        // TASK_END demonstrate
    });
}
