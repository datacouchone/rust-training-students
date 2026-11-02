use case06_admission::*;
use std::sync::{Arc, Barrier};
#[test]
fn tenant_rejection_rolls_back_global() {
    let g = Budget::new(100);
    let t = Budget::new(4);
    assert!(matches!(Lease::reserve(&g, &t, 5), Err(Rejected::Tenant)));
    assert_eq!(g.used(), 0);
    assert_eq!(t.used(), 0);
}
#[test]
fn global_rejection_does_not_touch_tenant() {
    let g = Budget::new(4);
    let t = Budget::new(100);
    assert!(matches!(Lease::reserve(&g, &t, 5), Err(Rejected::Global)));
    assert_eq!(t.used(), 0);
}
#[test]
fn exact_limit_zero_and_overflow() {
    let g = Budget::new(usize::MAX);
    let t = Budget::new(usize::MAX);
    let a = Lease::reserve(&g, &t, usize::MAX).unwrap();
    assert!(Lease::reserve(&g, &t, 1).is_err());
    let z = Lease::reserve(&g, &t, 0).unwrap();
    drop(z);
    assert_eq!(g.used(), usize::MAX);
    drop(a);
    assert_eq!(g.used(), 0);
    assert!(Lease::reserve(&g, &g, 0).is_err());
}
#[test]
fn unwind_releases_both_budgets() {
    let g = Budget::new(20);
    let t = Budget::new(20);
    let acquired = std::sync::atomic::AtomicBool::new(false);
    let _ = std::panic::catch_unwind(|| {
        let _l = Lease::reserve(&g, &t, 17).unwrap();
        acquired.store(true, std::sync::atomic::Ordering::Relaxed);
        panic!("request decode failure")
    });
    assert!(
        acquired.load(std::sync::atomic::Ordering::Relaxed),
        "reservation must succeed before unwinding"
    );
    assert_eq!(g.used(), 0);
    assert_eq!(t.used(), 0);
}
#[test]
fn concurrent_reservations_cannot_overbook() {
    let g = Budget::new(7);
    let t = Budget::new(5);
    let barrier = Arc::new(Barrier::new(17));
    let release = Arc::new(Barrier::new(17));
    let threads: Vec<_> = (0..16)
        .map(|_| {
            let (g, t, b, r) = (g.clone(), t.clone(), barrier.clone(), release.clone());
            std::thread::spawn(move || {
                let lease =
                    std::panic::catch_unwind(|| Lease::reserve(&g, &t, 1).ok()).unwrap_or(None);
                b.wait();
                r.wait();
                lease.is_some()
            })
        })
        .collect();
    barrier.wait();
    let held = (g.used(), t.used());
    release.wait();
    assert_eq!(
        threads
            .into_iter()
            .map(|h| usize::from(h.join().unwrap()))
            .sum::<usize>(),
        5
    );
    assert_eq!(held, (5, 5));
    assert_eq!(g.used(), 0);
    assert_eq!(t.used(), 0);
}
#[test]
fn tenants_share_global_limit_without_sharing_quota() {
    let g = Budget::new(10);
    let a = Budget::new(8);
    let b = Budget::new(8);
    let x = Lease::reserve(&g, &a, 8).unwrap();
    assert!(Lease::reserve(&g, &b, 3).is_err());
    let y = Lease::reserve(&g, &b, 2).unwrap();
    drop(x);
    assert_eq!(a.used(), 0);
    assert_eq!(b.used(), 2);
    assert_eq!(g.used(), 2);
    drop(y);
}
