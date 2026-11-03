use day04_challenge::{
    Request,
    gateway::Gateway,
    policy::{self, Ledger, Router},
    telemetry::{self, Buffer, Event, Summary},
    transport::{Fixture, request},
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
#[test]
fn rejected_admission_does_not_change_state() {
    let s = Arc::new(Mutex::new(Ledger {
        used: 0,
        accepted: 0,
        limit: 8,
    }));
    let lease = policy::admit(&s, 5).unwrap();
    assert!(policy::admit(&s, 4).is_none());
    let snapshot = {
        let state = s.lock().unwrap();
        (state.used, state.accepted)
    };
    assert_eq!(snapshot, (5, 1));
    drop(lease);
    assert_eq!(s.lock().unwrap().used, 0);
}
#[test]
fn retry_budget_stays_exhausted() {
    let budget = AtomicUsize::new(3);
    let admitted = (0..20).filter(|_| policy::take_retry(&budget)).count();
    assert_eq!(admitted, 3);
    assert_eq!(budget.load(Ordering::Relaxed), 0);
}
#[test]
fn shared_retry_budget_does_not_overspend() {
    let b = Arc::new(AtomicUsize::new(7));
    let threads: Vec<_> = (0..16)
        .map(|_| {
            let b = Arc::clone(&b);
            std::thread::spawn(move || (0..20).filter(|_| policy::take_retry(&b)).count())
        })
        .collect();
    assert_eq!(
        threads
            .into_iter()
            .map(|t| t.join().unwrap())
            .sum::<usize>(),
        7
    );
}
#[test]
fn query_preserves_validity_status_bounds_and_widening() {
    let e = [
        Event {
            latency: u32::MAX,
            status: 503,
            valid: true,
        },
        Event {
            latency: u32::MAX,
            status: 599,
            valid: true,
        },
        Event {
            latency: 42,
            status: 503,
            valid: false,
        },
        Event {
            latency: 99,
            status: 600,
            valid: true,
        },
        Event {
            latency: 7,
            status: 200,
            valid: true,
        },
    ];
    assert_eq!(
        telemetry::summarize(&e, 7),
        Summary {
            count: 2,
            total: u32::MAX as u64 * 2
        }
    );
}
#[test]
fn burst_capacity_is_released_and_small_buffers_reused() {
    let mut b = Buffer {
        bytes: vec![0; 60000],
        retain: 4096,
    };
    b.flush();
    assert!(b.bytes.is_empty());
    assert!(b.bytes.capacity() <= 4096);
    b.bytes.extend_from_slice(b"small");
    let cap = b.bytes.capacity();
    b.flush();
    assert_eq!(b.bytes.capacity(), cap);
}
#[test]
fn routing_keeps_segment_and_tenant_precedence() {
    let r = Router::new(50);
    for (tenant, path, expected) in [
        (7, "/api/orders/123", Some(17)),
        (9, "/api/orders", Some(1)),
        (7, "/apiary", Some(0)),
        (7, "bad", None),
        (7, "/", Some(0)),
    ] {
        assert_eq!(r.route(tenant, path), expected);
    }
}
#[test]
fn routing_work_depends_on_path_depth_not_policy_size() {
    let r = Router::new(4096);
    assert_eq!(r.route(7, "/api/orders/123"), Some(17));
    assert!(
        r.inspected.load(Ordering::Relaxed) <= 4,
        "count the candidate prefixes examined; preserve the counter"
    );
}
#[tokio::test(start_paused = true)]
async fn deadline_does_not_restart_after_queueing_or_retry() {
    let s = Gateway::new(1, 64, 1, Duration::from_millis(50));
    let start = tokio::time::Instant::now();
    tokio::time::advance(Duration::from_millis(30)).await;
    assert_eq!(s.deadline(start), start + Duration::from_millis(50));
}
#[tokio::test]
async fn timeout_does_not_leave_a_local_attempt_running() {
    let s = Gateway::new(1, 64, 0, Duration::from_millis(30));
    let f = Fixture::start(Arc::clone(&s), None).await.unwrap();
    let mut req = Request::sample(1);
    req.delay_ms = 500;
    let r = request(f.addr, &req).await.unwrap();
    assert_eq!(r.status, "timeout");
    assert_eq!(
        s.metrics.active.load(Ordering::SeqCst),
        0,
        "reply must not precede local attempt cleanup"
    );
    assert_eq!(s.ledger.lock().unwrap().used, 0);
    f.shutdown().await;
}
#[tokio::test]
async fn unrelated_requests_can_reach_upstream_together() {
    let s = Gateway::new(1, 64, 1, Duration::from_secs(3));
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let f = Fixture::start(Arc::clone(&s), Some(Arc::clone(&gate)))
        .await
        .unwrap();
    let addr = f.addr;
    let a = tokio::spawn(async move { request(addr, &Request::sample(1)).await });
    let b = tokio::spawn(async move { request(addr, &Request::sample(2)).await });
    let overlap = tokio::time::timeout(Duration::from_secs(1), async {
        while f.arrivals.load(Ordering::SeqCst) < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .is_ok();
    gate.add_permits(128);
    let _ = a.await;
    let _ = b.await;
    f.shutdown().await;
    assert!(
        overlap,
        "both requests must reach the gated upstream before either is released"
    );
}

#[tokio::test]
async fn deadline_includes_waiting_for_policy_access() {
    let s = Gateway::new(1, 64, 1, Duration::from_millis(30));
    let f = Fixture::start(Arc::clone(&s), None).await.unwrap();
    let guard = s.router.lock().await;
    let addr = f.addr;
    let mut task = tokio::spawn(async move { request(addr, &Request::sample(1)).await });
    let result = tokio::time::timeout(Duration::from_millis(300), &mut task).await;
    drop(guard);
    let completed = result.is_ok();
    if let Ok(reply) = result {
        assert_eq!(reply.unwrap().unwrap().status, "timeout");
    } else {
        let _ = task.await;
    }
    f.shutdown().await;
    assert!(
        completed,
        "queue wait must be covered by the request deadline"
    );
}
#[tokio::test]
async fn parent_cancellation_releases_attempt_and_lease() {
    let s = Gateway::new(1, 64, 1, Duration::from_secs(2));
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let f = Fixture::start(Arc::clone(&s), Some(Arc::clone(&gate)))
        .await
        .unwrap();
    let service = Arc::clone(&s);
    let origin = f.upstream;
    let task = tokio::spawn(async move { service.handle(Request::sample(1), origin).await });
    tokio::time::timeout(Duration::from_secs(1), async {
        while f.arrivals.load(Ordering::SeqCst) < 1 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    task.abort();
    let _ = task.await;
    let active = s.metrics.active.load(Ordering::SeqCst);
    let used = s.ledger.lock().unwrap().used;
    gate.add_permits(128);
    f.shutdown().await;
    assert_eq!(active, 0);
    assert_eq!(used, 0);
}
