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
        let ids = [0usize, 1, 2, 3];
        let done = tokio::spawn(async { 0usize }).await.unwrap();
        let mut outcomes = BTreeMap::from([(done, "Done")]);
        let mut tasks = JoinSet::new();
        let (tx, mut rx) = mpsc::channel(2);
        for id in [1usize, 2] { let tx = tx.clone(); tasks.spawn(async move { tx.send(id).await.unwrap(); std::future::pending::<()>().await; }); }
        for _ in 0..2 { rx.recv().await.unwrap(); }
        tasks.abort_all();
        while let Some(result) = tasks.join_next().await { assert!(result.unwrap_err().is_cancelled()); }
        for id in [1usize, 2] { outcomes.insert(id, "Cancelled"); }
        let missing = ids.len() - outcomes.len();
        assert_eq!(missing, 1);
        println!("outcomes={outcomes:?}; missing={missing}");
        // TASK_END demonstrate
    });
}
