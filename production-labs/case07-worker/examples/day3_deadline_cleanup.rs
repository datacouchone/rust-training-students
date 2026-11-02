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
        let (mut handle, started, stopped) = parked(&active);
        started.await.unwrap();
        let start = time::Instant::now();
        let result = time::timeout(Duration::from_secs(5), &mut handle).await;
        assert!(result.is_err());
        assert!(start.elapsed() >= Duration::from_secs(5));
        assert_eq!(count(&active), 1);
        println!("deadline=true; active after timeout={}", count(&active));
        handle.abort();
        let _ = handle.await;
        stopped.await.unwrap();
        // TASK_END demonstrate
    });
}
