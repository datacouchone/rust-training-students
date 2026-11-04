use case06c_reserve::*;
use std::{
    sync::{Arc, Barrier, mpsc},
    thread,
    time::Duration,
};
fn pool() -> Arc<Pool> {
    Pool::new(Config {
        capacity: 10,
        critical_reserve: 3,
    })
    .unwrap()
}
#[test]
fn reserve_is_for_critical_traffic_but_critical_can_borrow_all() {
    let p = pool();
    let n = p.try_acquire(Class::Normal, 7).unwrap();
    assert_eq!(p.try_acquire(Class::Normal, 1).unwrap_err(), Error::Busy);
    let c = p.try_acquire(Class::Critical, 3).unwrap();
    assert_eq!(p.snapshot().normal + p.snapshot().critical, 10);
    drop((n, c));
    let all = p.try_acquire(Class::Critical, 10).unwrap();
    assert_eq!(p.try_acquire(Class::Normal, 1).unwrap_err(), Error::Busy);
    drop(all);
}
#[test]
fn rejection_is_transactional_and_overflow_never_wraps() {
    let p = pool();
    let n = p.try_acquire(Class::Normal, 5).unwrap();
    let before = p.snapshot();
    assert_eq!(
        p.try_acquire(Class::Normal, 0).unwrap_err(),
        Error::ZeroWeight
    );
    assert_eq!(
        p.try_acquire(Class::Normal, usize::MAX).unwrap_err(),
        Error::TooLarge
    );
    assert_eq!(p.try_acquire(Class::Critical, 6).unwrap_err(), Error::Busy);
    assert_eq!(p.snapshot(), before);
    drop(n);
    let p = Pool::new(Config {
        capacity: usize::MAX,
        critical_reserve: 0,
    })
    .unwrap();
    let a = p.try_acquire(Class::Critical, usize::MAX).unwrap();
    assert_eq!(p.try_acquire(Class::Normal, 1).unwrap_err(), Error::Busy);
    assert_eq!(p.snapshot().critical, usize::MAX);
    drop(a);
}
#[test]
fn reconfiguration_is_atomic_and_preserves_live_lease_accounting() {
    let p = pool();
    let n = p.try_acquire(Class::Normal, 7).unwrap();
    let before = p.snapshot();
    assert_eq!(
        p.reconfigure(Config {
            capacity: 10,
            critical_reserve: 4
        }),
        Err(Error::WouldOvercommit)
    );
    assert_eq!(p.snapshot(), before);
    assert_eq!(
        p.reconfigure(Config {
            capacity: 6,
            critical_reserve: 0
        }),
        Err(Error::WouldOvercommit)
    );
    assert_eq!(p.snapshot(), before);
    assert_eq!(
        p.reconfigure(Config {
            capacity: 2,
            critical_reserve: 3
        }),
        Err(Error::InvalidConfig)
    );
    assert_eq!(p.snapshot(), before);
    p.reconfigure(Config {
        capacity: 20,
        critical_reserve: 5,
    })
    .unwrap();
    drop(n);
    assert_eq!(p.snapshot().normal, 0);
    let all = p.try_acquire(Class::Normal, 15).unwrap();
    drop(all);
}
#[test]
fn close_is_terminal_and_existing_leases_still_release() {
    let p = pool();
    let n = p.try_acquire(Class::Normal, 7).unwrap();
    p.close();
    p.close();
    assert_eq!(
        p.try_acquire(Class::Critical, 1).unwrap_err(),
        Error::Closed
    );
    assert_eq!(
        p.reconfigure(Config {
            capacity: 20,
            critical_reserve: 0
        }),
        Err(Error::Closed)
    );
    drop(n);
    assert_eq!(p.snapshot().normal, 0);
}
#[test]
fn contenders_cannot_overbook_while_leases_are_held() {
    let p = pool();
    let start = Arc::new(Barrier::new(17));
    let hold = Arc::new(Barrier::new(17));
    let (tx, rx) = mpsc::channel();
    let mut workers = vec![];
    for i in 0..16 {
        let (p, start, hold, tx) = (p.clone(), start.clone(), hold.clone(), tx.clone());
        workers.push(thread::spawn(move || {
            start.wait();
            let class = if i % 3 == 0 {
                Class::Critical
            } else {
                Class::Normal
            };
            let lease = p.try_acquire(class, 1).ok();
            tx.send((class, lease.is_some())).unwrap();
            hold.wait();
            drop(lease);
        }));
    }
    start.wait();
    let outcomes: Vec<_> = (0..16)
        .map(|_| rx.recv_timeout(Duration::from_secs(2)).unwrap())
        .collect();
    let s = p.snapshot();
    hold.wait();
    for w in workers {
        w.join().unwrap();
    }
    let normal = outcomes
        .iter()
        .filter(|(c, yes)| *c == Class::Normal && *yes)
        .count();
    let critical = outcomes
        .iter()
        .filter(|(c, yes)| *c == Class::Critical && *yes)
        .count();
    assert_eq!((s.normal, s.critical), (normal, critical));
    assert!(normal <= 7);
    assert!(normal + critical <= 10);
    assert_eq!(p.snapshot().normal + p.snapshot().critical, 0);
}
#[test]
fn waiting_request_rechecks_after_capacity_is_returned() {
    let p = pool();
    let held = p.try_acquire(Class::Critical, 10).unwrap();
    let (tx, rx) = mpsc::channel();
    let q = p.clone();
    let w = thread::spawn(move || {
        tx.send(q.wait_acquire(Class::Normal, 7, Duration::from_secs(2)))
            .unwrap()
    });
    assert!(rx.recv_timeout(Duration::from_millis(30)).is_err());
    drop(held);
    let acquired = rx.recv_timeout(Duration::from_secs(3)).unwrap().unwrap();
    w.join().unwrap();
    assert_eq!(p.snapshot().normal, 7);
    drop(acquired);
}
#[test]
fn close_wakes_waiters_without_waiting_for_lease_release() {
    let p = pool();
    let held = p.try_acquire(Class::Critical, 10).unwrap();
    let (tx, rx) = mpsc::channel();
    let q = p.clone();
    let w = thread::spawn(move || {
        tx.send(q.wait_acquire(Class::Critical, 1, Duration::from_secs(2)))
            .unwrap()
    });
    assert!(rx.recv_timeout(Duration::from_millis(30)).is_err());
    p.close();
    let result = rx.recv_timeout(Duration::from_secs(1));
    drop(held);
    w.join().unwrap();
    assert_eq!(result.unwrap().unwrap_err(), Error::Closed);
}
#[test]
fn increasing_capacity_notifies_waiters() {
    let p = pool();
    let held = p.try_acquire(Class::Critical, 10).unwrap();
    let (tx, rx) = mpsc::channel();
    let q = p.clone();
    let w = thread::spawn(move || {
        tx.send(q.wait_acquire(Class::Normal, 2, Duration::from_secs(2)))
            .unwrap()
    });
    assert!(rx.recv_timeout(Duration::from_millis(30)).is_err());
    p.reconfigure(Config {
        capacity: 12,
        critical_reserve: 3,
    })
    .unwrap();
    let result = rx.recv_timeout(Duration::from_secs(1));
    drop(held);
    w.join().unwrap();
    let lease = result.unwrap().unwrap();
    drop(lease);
}
#[test]
fn timeout_is_one_budget_despite_repeated_notifications() {
    let p = pool();
    let held = p.try_acquire(Class::Critical, 10).unwrap();
    let (tx, rx) = mpsc::channel();
    let q = p.clone();
    let w = thread::spawn(move || {
        tx.send(q.wait_acquire(Class::Normal, 1, Duration::from_millis(60)))
            .unwrap()
    });
    // Keep notifying longer than the entire timeout. Incorrect timeout reset would outlive this window.
    for _ in 0..16 {
        p.reconfigure(p.snapshot().config).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    let result = rx.try_recv();
    drop(held);
    w.join().unwrap();
    assert_eq!(result.unwrap().unwrap_err(), Error::Timeout);
}
#[test]
fn unwind_and_zero_wait_preserve_ownership() {
    let p = pool();
    let q = p.clone();
    let _ = std::panic::catch_unwind(move || {
        let _lease = q.try_acquire(Class::Normal, 7).unwrap();
        panic!("client panic outside pool lock")
    });
    assert_eq!(p.snapshot().normal, 0);
    let lease = p.wait_acquire(Class::Normal, 7, Duration::ZERO).unwrap();
    assert_eq!(
        p.wait_acquire(Class::Normal, 1, Duration::ZERO)
            .unwrap_err(),
        Error::Timeout
    );
    drop(lease);
}
