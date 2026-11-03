use day05_hackathon::{
    analytics::*,
    boundary::*,
    ledger::*,
    rollout::*,
    worker::{Error, Service},
};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tokio::time::{Instant, timeout};
fn event(t: u32, id: u64, n: u64, p: &str) -> Validated {
    Validated::try_from(Untrusted(encode(t, id, n, p))).unwrap()
}
#[test]
fn ffi_equality_is_complete_not_prefix() {
    assert!(bytes_equal(b"abc", b"abc"));
    assert!(!bytes_equal(b"abc", b"abcd"));
    assert!(!bytes_equal(b"", b"x"));
    assert!(bytes_equal(b"", b""));
}
#[test]
fn boundary_rejects_unknown_version() {
    let mut b = encode(1, 1, 2, "/a");
    b[0] = 2;
    assert_eq!(
        Validated::try_from(Untrusted(b)).unwrap_err(),
        Invalid::Version
    );
}
#[test]
fn boundary_rejects_relative_and_non_utf8_path() {
    assert!(Validated::try_from(Untrusted(encode(1, 1, 1, "relative"))).is_err());
    let mut b = encode(1, 1, 1, "/");
    b.push(255);
    assert!(Validated::try_from(Untrusted(b)).is_err());
}
#[test]
fn boundary_accepts_unaligned_wire_slice_and_unicode() {
    let b = encode(7, 8, 9, "/λ");
    let mut storage = vec![255];
    storage.extend(&b);
    let e = Validated::try_from(Untrusted(storage[1..].to_vec())).unwrap();
    assert_eq!((e.tenant(), e.id(), e.bytes()), (7, 8, 9));
}
#[test]
fn replay_after_lost_ack_does_not_charge_twice() {
    let mut l = Ledger::new(2, 3);
    let e = event(1, 9, 100, "/a");
    assert_eq!(l.apply(&e), Ok(Outcome::Applied));
    assert_eq!(l.apply(&e), Ok(Outcome::Replay));
    assert_eq!(
        l.accounts[&1],
        Account {
            count: 1,
            bytes: 100
        }
    );
}
#[test]
fn changed_body_under_same_id_is_conflict() {
    let mut l = Ledger::new(2, 3);
    l.apply(&event(1, 9, 1, "/a")).unwrap();
    let old = l.accounts.clone();
    assert_eq!(l.apply(&event(1, 9, 1, "/ab")), Err(Reject::Conflict));
    assert_eq!(l.accounts, old);
}
#[test]
fn rejection_does_not_mutate_account_or_receipts() {
    let mut l = Ledger::new(2, 3);
    l.apply(&event(1, 1, u64::MAX, "/a")).unwrap();
    let old = (l.accounts.clone(), l.receipts.clone());
    assert_eq!(l.apply(&event(1, 2, 1, "/a")), Err(Reject::Overflow));
    assert_eq!((l.accounts, l.receipts), old);
}
#[test]
fn tenant_bound_and_receipt_bound_are_independent() {
    let mut l = Ledger::new(1, 2);
    l.apply(&event(1, 1, 1, "/a")).unwrap();
    assert_eq!(l.apply(&event(2, 1, 1, "/a")), Err(Reject::Full));
    l.apply(&event(1, 2, 1, "/a")).unwrap();
    assert_eq!(l.apply(&event(1, 3, 1, "/a")), Err(Reject::Full));
    assert_eq!(l.apply(&event(1, 1, 1, "/a")), Ok(Outcome::Replay));
}
#[test]
fn request_identity_is_scoped_to_tenant() {
    let mut l = Ledger::new(2, 3);
    l.apply(&event(1, 7, 10, "/a")).unwrap();
    l.apply(&event(2, 7, 20, "/a")).unwrap();
    assert_eq!(l.accounts[&2].bytes, 20);
}
#[tokio::test(start_paused = true)]
async fn admission_wait_obeys_original_deadline() {
    let s = Service::new(1, 2, 3);
    let hold = s.slots.clone().acquire_owned().await.unwrap();
    let result = timeout(
        Duration::from_millis(100),
        s.submit(
            event(1, 1, 1, "/a"),
            Instant::now() + Duration::from_millis(10),
            Duration::ZERO,
        ),
    )
    .await;
    drop(hold);
    assert_eq!(result.unwrap(), Err(Error::Timeout));
}
#[tokio::test(start_paused = true)]
async fn timeout_cancels_owned_work_and_releases_slot() {
    let s = Service::new(1, 2, 3);
    let result = s
        .submit(
            event(1, 1, 1, "/a"),
            Instant::now() + Duration::from_millis(10),
            Duration::from_secs(1),
        )
        .await;
    assert_eq!(result, Err(Error::Timeout));
    assert_eq!(s.active.load(Ordering::SeqCst), 0);
    assert_eq!(s.slots.available_permits(), 1);
    assert!(s.ledger.lock().await.accounts.is_empty());
}
#[tokio::test(start_paused = true)]
async fn parent_cancellation_owns_child_lifetime() {
    let s = Service::new(1, 2, 3);
    let copy = Arc::clone(&s);
    let task = tokio::spawn(async move {
        copy.submit(
            event(1, 1, 1, "/a"),
            Instant::now() + Duration::from_secs(5),
            Duration::from_secs(1),
        )
        .await
    });
    for _ in 0..10 {
        tokio::task::yield_now().await;
        if s.active.load(Ordering::SeqCst) > 0 {
            break;
        }
    }
    assert_eq!(s.active.load(Ordering::SeqCst), 1);
    task.abort();
    let _ = task.await;
    assert_eq!(s.active.load(Ordering::SeqCst), 0);
    assert_eq!(s.slots.available_permits(), 1);
}
#[tokio::test(start_paused = true)]
async fn enrichment_wait_does_not_serialize_ledger_access() {
    let s = Service::new(2, 2, 3);
    let start = Instant::now();
    let deadline = start + Duration::from_secs(1);
    let (a, b) = tokio::join!(
        s.submit(event(1, 1, 1, "/a"), deadline, Duration::from_millis(50)),
        s.submit(event(1, 2, 1, "/a"), deadline, Duration::from_millis(50))
    );
    assert_eq!(a, Ok(Outcome::Applied));
    assert_eq!(b, Ok(Outcome::Applied));
    assert!(
        start.elapsed() < Duration::from_millis(75),
        "independent enrichment should overlap"
    );
}
#[tokio::test(start_paused = true)]
async fn concurrent_replays_commit_once() {
    let s = Service::new(8, 2, 8);
    let mut jobs = tokio::task::JoinSet::new();
    for _ in 0..8 {
        let s = Arc::clone(&s);
        jobs.spawn(async move {
            s.submit(
                event(1, 42, 10, "/a"),
                Instant::now() + Duration::from_secs(1),
                Duration::ZERO,
            )
            .await
        });
    }
    while let Some(r) = jobs.join_next().await {
        r.unwrap().unwrap();
    }
    assert_eq!(
        s.ledger.lock().await.accounts[&1],
        Account {
            count: 1,
            bytes: 10
        }
    );
}
#[test]
fn query_uses_validity_upper_bound_and_wide_sum() {
    let rows = [
        Row {
            valid: true,
            status: 503,
            latency: u32::MAX,
        },
        Row {
            valid: true,
            status: 599,
            latency: u32::MAX,
        },
        Row {
            valid: false,
            status: 503,
            latency: 99,
        },
        Row {
            valid: true,
            status: 600,
            latency: 99,
        },
    ];
    assert_eq!(
        summarize(&rows, 0),
        Summary {
            count: 2,
            total: 2 * u64::from(u32::MAX)
        }
    );
}
#[test]
fn burst_releases_large_capacity_but_reuses_small() {
    let mut b = Buffer {
        bytes: vec![0; 65536],
        retain: 4096,
    };
    b.flush();
    assert!(b.bytes.capacity() <= 4096);
    let mut b = Buffer {
        bytes: Vec::with_capacity(16),
        retain: 4096,
    };
    b.bytes.push(1);
    b.flush();
    assert_eq!(b.bytes.len(), 0);
    assert_eq!(b.bytes.capacity(), 16);
}
fn healthy() -> Window {
    Window {
        requests: 1000,
        successes: 990,
        errors: 10,
        p95_us: 50_000,
        duplicates: 0,
    }
}
#[test]
fn rollout_holds_missing_small_and_inconsistent_evidence() {
    assert_eq!(
        decide(Window {
            requests: 0,
            successes: 0,
            errors: 0,
            ..healthy()
        }),
        Decision::Hold
    );
    assert_eq!(
        decide(Window {
            requests: 10,
            successes: 10,
            errors: 0,
            ..healthy()
        }),
        Decision::Hold
    );
    assert_eq!(
        decide(Window {
            successes: 1000,
            ..healthy()
        }),
        Decision::Hold
    );
}
#[test]
fn rollout_never_promotes_duplicate_effects() {
    assert_eq!(
        decide(Window {
            duplicates: 1,
            ..healthy()
        }),
        Decision::Rollback
    );
}
#[test]
fn rollout_thresholds_preserve_boundary() {
    assert_eq!(decide(healthy()), Decision::Promote);
    assert_eq!(
        decide(Window {
            successes: 989,
            errors: 11,
            ..healthy()
        }),
        Decision::Rollback
    );
    assert_eq!(
        decide(Window {
            p95_us: 50_001,
            ..healthy()
        }),
        Decision::Rollback
    );
}

use day05_hackathon::codec::*;
use std::io::ErrorKind;
use tokio::io::{AsyncWriteExt, duplex};
#[tokio::test(start_paused = true)]
async fn cancellation_after_every_byte_preserves_frame() {
    let bytes = frame(b"12:100");
    for split in 1..bytes.len() {
        let (mut tx, mut rx) = duplex(64);
        let mut d = Decoder::default();
        tx.write_all(&bytes[..split]).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(1), d.next(&mut rx))
                .await
                .is_err()
        );
        assert_eq!(d.buffered(), split);
        tx.write_all(&bytes[split..]).await.unwrap();
        drop(tx);
        assert_eq!(d.next(&mut rx).await.unwrap(), Some(b"12:100".to_vec()));
    }
}
#[tokio::test]
async fn back_to_back_empty_and_clean_eof() {
    let (mut tx, mut rx) = duplex(64);
    tx.write_all(&frame(b"")).await.unwrap();
    tx.write_all(&frame(b"1:9")).await.unwrap();
    drop(tx);
    let mut d = Decoder::default();
    assert_eq!(d.next(&mut rx).await.unwrap(), Some(vec![]));
    assert_eq!(d.next(&mut rx).await.unwrap(), Some(b"1:9".to_vec()));
    assert_eq!(d.next(&mut rx).await.unwrap(), None);
}
#[tokio::test]
async fn rejects_length_before_body_allocation_and_poisoned_retry() {
    let (mut tx, mut rx) = duplex(64);
    tx.write_all(&u32::MAX.to_be_bytes()).await.unwrap();
    let mut d = Decoder::default();
    assert_eq!(
        d.next(&mut rx).await.unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(d.buffered(), 4);
    assert_eq!(
        tokio::time::timeout(Duration::from_millis(100), d.next(&mut rx))
            .await
            .expect("poisoned stream must fail immediately")
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
}
#[tokio::test]
async fn every_truncated_frame_is_distinct_from_clean_eof() {
    let bytes = frame(b"1:2");
    for n in 1..bytes.len() {
        let (mut tx, mut rx) = duplex(64);
        tx.write_all(&bytes[..n]).await.unwrap();
        drop(tx);
        assert_eq!(
            Decoder::default().next(&mut rx).await.unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );
    }
}
