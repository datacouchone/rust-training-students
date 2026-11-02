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
        let mut tasks = JoinSet::new();
        let mut max_tracked = 0;
        for id in 0..4 {
        tasks.spawn(async move { time::sleep(Duration::from_secs(1)).await; id });
        max_tracked = max_tracked.max(tasks.len());
        }
        let mut completed = 0;
        while let Some(result) = tasks.join_next().await { result.unwrap(); completed += 1; }
        assert_eq!((max_tracked, completed), (4, 4));
        println!("max spawned/unjoined={max_tracked}; completed={completed}; limit=2");
        // TASK_END demonstrate
    });
}
