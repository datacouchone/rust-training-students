use case07_worker::*;
use std::{sync::Arc, time::Duration};
use tokio::sync::{oneshot, watch};
fn jobs(n: u64) -> Vec<Job> {
    (0..n)
        .map(|id| Job {
            id,
            latency: Duration::from_millis(10),
            fail: id == 3,
        })
        .collect()
}
#[tokio::test(start_paused = true)]
async fn bounded_fanout_and_all_outcomes() {
    let (_tx, rx) = watch::channel(false);
    let p = Arc::new(Probe::default());
    let r = run_batch(jobs(20), 3, Duration::from_secs(1), rx, p.clone())
        .await
        .unwrap();
    assert_eq!(r.len(), 20);
    assert_eq!(r[&3], Outcome::Failed);
    assert_eq!(r[&19], Outcome::Done(19));
    assert_eq!(p.peak(), 3);
    assert_eq!(p.active(), 0);
}
#[tokio::test(start_paused = true)]
async fn deadlines_release_slots() {
    let (_tx, rx) = watch::channel(false);
    let p = Arc::new(Probe::default());
    let r = run_batch(jobs(20), 2, Duration::from_millis(1), rx, p.clone())
        .await
        .unwrap();
    assert!(r.values().all(|v| *v == Outcome::TimedOut));
    assert_eq!(p.active(), 0);
    assert_eq!(p.peak(), 2);
}
#[tokio::test(start_paused = true)]
async fn cancellation_reconciles_queued_and_running() {
    let (tx, rx) = watch::channel(false);
    let p = Arc::new(Probe::default());
    let pp = p.clone();
    let handle = tokio::spawn(run_batch(jobs(20), 2, Duration::from_secs(1), rx, pp));
    for _ in 0..100 {
        if p.active() == 2 {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(p.active(), 2, "worker did not admit the expected jobs");
    tx.send(true).unwrap();
    let r = handle.await.unwrap().unwrap();
    assert_eq!(r.len(), 20);
    assert!(r.values().all(|v| *v == Outcome::Cancelled));
    assert_eq!(p.active(), 0);
}
#[tokio::test(start_paused = true)]
async fn already_cancelled_admits_nothing() {
    let (_tx, rx) = watch::channel(true);
    let p = Arc::new(Probe::default());
    let r = run_batch(jobs(5), 2, Duration::from_secs(1), rx, p.clone())
        .await
        .unwrap();
    assert_eq!(p.peak(), 0);
    assert_eq!(r.len(), 5);
}
struct Signal(Option<oneshot::Sender<()>>);
impl Drop for Signal {
    fn drop(&mut self) {
        if let Some(tx) = self.0.take() {
            let _ = tx.send(());
        }
    }
}
#[tokio::test(start_paused = true)]
async fn timed_out_join_does_not_detach_task() {
    let (tx, mut rx) = oneshot::channel();
    let handle = tokio::spawn(async move {
        let _guard = Signal(Some(tx));
        std::future::pending::<()>().await;
    });
    assert_eq!(
        join_deadline(handle, Duration::from_millis(1)).await,
        Err(JoinFailure::Deadline)
    );
    assert_eq!(rx.try_recv(), Ok(()));
}
#[tokio::test(start_paused = true)]
async fn parent_drop_aborts_owned_join() {
    let (tx, rx) = oneshot::channel();
    let (start, ready) = oneshot::channel();
    let child = tokio::spawn(async move {
        let _g = Signal(Some(tx));
        start.send(()).unwrap();
        std::future::pending::<()>().await;
    });
    ready.await.unwrap();
    let parent = tokio::spawn(join_deadline(child, Duration::from_secs(50)));
    tokio::task::yield_now().await;
    parent.abort();
    let _ = parent.await;
    tokio::time::timeout(Duration::from_secs(1), rx)
        .await
        .expect("child was detached")
        .unwrap();
}
#[tokio::test]
async fn validates_batch_before_spawning() {
    let (_tx, rx) = watch::channel(false);
    let p = Arc::new(Probe::default());
    assert_eq!(
        run_batch(jobs(1), 0, Duration::ZERO, rx.clone(), p.clone()).await,
        Err(Error::ZeroLimit)
    );
    let j = Job {
        id: 1,
        latency: Duration::ZERO,
        fail: false,
    };
    assert_eq!(
        run_batch(vec![j.clone(), j], 2, Duration::ZERO, rx.clone(), p.clone()).await,
        Err(Error::DuplicateId)
    );
    assert_eq!(
        run_batch(jobs(MAX_BATCH as u64 + 1), 2, Duration::ZERO, rx, p.clone()).await,
        Err(Error::TooMany)
    );
    assert_eq!(p.peak(), 0);
}

#[tokio::test(start_paused = true)]
async fn dropping_unpolled_deadline_future_still_aborts_child() {
    let (tx, rx) = oneshot::channel();
    let (start, ready) = oneshot::channel();
    let child = tokio::spawn(async move {
        let _g = Signal(Some(tx));
        start.send(()).unwrap();
        std::future::pending::<()>().await;
    });
    ready.await.unwrap();
    let wrapper = join_deadline(child, Duration::from_secs(50));
    drop(wrapper);
    tokio::time::timeout(Duration::from_secs(1), rx)
        .await
        .expect("unpolled wrapper detached child")
        .unwrap();
}
