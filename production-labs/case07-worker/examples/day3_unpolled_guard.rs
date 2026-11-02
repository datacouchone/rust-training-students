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
        let (handle, started, stopped) = parked(&active);
        let emergency_abort = handle.abort_handle();
        started.await.unwrap();
        let wrapper = async move {
        let mut owner = AbortOnDrop(handle);
        let _ = (&mut owner.0).await;
        };
        drop(wrapper);
        assert_eq!(count(&active), 1);
        println!("unpolled wrapper dropped: active={}", count(&active));
        emergency_abort.abort();
        stopped.await.unwrap();
        // TASK_END demonstrate
    });
}
