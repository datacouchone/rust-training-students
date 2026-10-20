use miri_puzzle::{Audit, Batch, OneShot};
use std::{
    cell::Cell,
    sync::{Arc, Barrier, Mutex},
    thread,
};

#[derive(Debug)]
struct Tracked {
    id: usize,
    drops: Arc<Mutex<Vec<usize>>>,
}
impl Drop for Tracked {
    fn drop(&mut self) {
        self.drops.lock().unwrap().push(self.id);
    }
}
fn tracked(id: usize, drops: &Arc<Mutex<Vec<usize>>>) -> Tracked {
    Tracked {
        id,
        drops: Arc::clone(drops),
    }
}
fn destroyed(drops: &Arc<Mutex<Vec<usize>>>) -> Vec<usize> {
    let mut ids = drops.lock().unwrap().clone();
    ids.sort_unstable();
    ids
}

#[test]
fn batch_bounds_include_empty_one_past_and_huge_indices() {
    let mut b = Batch::<u32, 4>::new();
    assert_eq!(b.get(0), None);
    assert_eq!(b.get(usize::MAX), None);
    b.push(10).unwrap();
    b.push(20).unwrap();
    assert_eq!(b.get(0), Some(&10));
    assert_eq!(b.get(1), Some(&20));
    assert_eq!(b.get(2), None); // spare capacity is not an initialized element
    assert_eq!(b.get(4), None);
    assert_eq!(b.get(usize::MAX), None);
}

#[test]
fn batch_full_rejection_returns_the_original_owner() {
    let drops = Arc::new(Mutex::new(Vec::new()));
    let mut b = Batch::<Tracked, 1>::new();
    b.push(tracked(0, &drops)).unwrap();
    let rejected = b.push(tracked(1, &drops)).unwrap_err();
    assert_eq!(rejected.id, 1);
    assert!(destroyed(&drops).is_empty());
    assert_eq!(b.len(), 1);
    drop(rejected);
    drop(b);
    assert_eq!(destroyed(&drops), [0, 1]);
}

#[test]
fn batch_pop_transfers_ownership_and_reuses_only_the_vacated_slot() {
    let drops = Arc::new(Mutex::new(Vec::new()));
    let mut b = Batch::<Tracked, 4>::new();
    b.push(tracked(0, &drops)).unwrap();
    b.push(tracked(1, &drops)).unwrap();
    let value = b.pop().unwrap();
    assert_eq!((value.id, b.len()), (1, 1));
    assert_eq!(b.get(1).map(|v| v.id), None);
    assert!(destroyed(&drops).is_empty());
    b.push(tracked(2, &drops)).unwrap();
    drop(b);
    assert_eq!(destroyed(&drops), [0, 2]);
    drop(value);
    assert_eq!(destroyed(&drops), [0, 1, 2]);
}

#[test]
fn batch_zero_capacity_and_zero_sized_values_are_valid() {
    let mut zero = Batch::<String, 0>::new();
    assert_eq!(
        zero.push(String::from("returned")),
        Err(String::from("returned"))
    );
    assert_eq!(zero.pop(), None);
    assert_eq!(zero.get(0), None);
    let mut zst = Batch::<(), 2>::new();
    zst.push(()).unwrap();
    zst.push(()).unwrap();
    assert_eq!(zst.push(()), Err(()));
    assert_eq!(zst.pop(), Some(()));
    assert_eq!(zst.pop(), Some(()));
    assert_eq!(zst.pop(), None);
}

#[test]
fn batch_partial_initialization_destroys_only_the_live_prefix() {
    let drops = Arc::new(Mutex::new(Vec::new()));
    let mut b = Batch::<Tracked, 8>::new();
    for id in 0..3 {
        b.push(tracked(id, &drops)).unwrap();
    }
    drop(b);
    assert_eq!(destroyed(&drops), [0, 1, 2]);
}

#[test]
fn batch_allocation_remains_owned_after_move_without_copying_payload() {
    let mut b = Batch::<String, 4>::new();
    b.push(String::from("snapshot")).unwrap();
    let address = b.get(0).unwrap().as_ptr();
    let moved = b;
    assert_eq!(moved.get(0).unwrap(), "snapshot");
    assert_eq!(moved.get(0).unwrap().as_ptr(), address);
}

#[test]
fn oneshot_empty_take_is_not_a_reservation_or_permanent_close() {
    let cell = OneShot::<String>::new();
    assert_eq!(cell.try_take(), None);
    assert_eq!(cell.try_publish(String::from("later")), Ok(()));
    assert_eq!(cell.try_take(), Some(String::from("later")));
    assert_eq!(cell.try_take(), None);
    assert_eq!(
        cell.try_publish(String::from("again")),
        Err(String::from("again"))
    );
}

#[test]
fn oneshot_unclaimed_and_transferred_values_drop_exactly_once() {
    let drops = Arc::new(Mutex::new(Vec::new()));
    {
        let cell = OneShot::new();
        cell.try_publish(tracked(0, &drops)).unwrap();
    }
    assert_eq!(destroyed(&drops), [0]);
    let cell = OneShot::new();
    cell.try_publish(tracked(1, &drops)).unwrap();
    let value = cell.try_take().unwrap();
    drop(cell);
    assert_eq!(destroyed(&drops), [0]);
    drop(value);
    assert_eq!(destroyed(&drops), [0, 1]);
}

#[test]
fn oneshot_competing_publishers_return_all_losing_owners() {
    let drops = Arc::new(Mutex::new(Vec::new()));
    let cell = OneShot::new();
    let gate = Barrier::new(6);
    let wins = Mutex::new(Vec::new());
    thread::scope(|s| {
        for id in 0..6 {
            let (cell, gate, wins, drops) = (&cell, &gate, &wins, &drops);
            s.spawn(move || {
                let value = tracked(id, drops);
                gate.wait();
                match cell.try_publish(value) {
                    Ok(()) => wins.lock().unwrap().push(id),
                    Err(original) => {
                        assert_eq!(original.id, id);
                        drop(original);
                    }
                }
            });
        }
    });
    assert_eq!(wins.lock().unwrap().len(), 1);
    let value = cell.try_take().unwrap();
    assert_eq!(value.id, wins.lock().unwrap()[0]);
    drop(value);
    drop(cell);
    assert_eq!(destroyed(&drops), [0, 1, 2, 3, 4, 5]);
}

#[test]
fn oneshot_competing_takers_transfer_only_one_owner() {
    let drops = Arc::new(Mutex::new(Vec::new()));
    let cell = OneShot::new();
    cell.try_publish(tracked(7, &drops)).unwrap();
    let gate = Barrier::new(6);
    let wins = Mutex::new(Vec::new());
    thread::scope(|s| {
        for _ in 0..6 {
            let (cell, gate, wins) = (&cell, &gate, &wins);
            s.spawn(move || {
                gate.wait();
                if let Some(value) = cell.try_take() {
                    wins.lock().unwrap().push(value.id);
                    drop(value);
                }
            });
        }
    });
    assert_eq!(*wins.lock().unwrap(), [7]);
    drop(cell);
    assert_eq!(destroyed(&drops), [7]);
}

#[test]
fn publication_before_join_must_make_payload_visible() {
    // The read occurs BEFORE joining the producer. A join before the read could
    // supply synchronization that hides a missing publication ordering.
    let cell = OneShot::new();
    thread::scope(|s| {
        s.spawn(|| {
            let mut batch = Batch::<String, 4>::new();
            batch.push(String::from("header")).unwrap();
            batch.push(String::from("payload")).unwrap();
            assert!(cell.try_publish(batch).is_ok());
        });
        let mut received = None;
        for _ in 0..2048 {
            if let Some(batch) = cell.try_take() {
                received = Some(batch);
                break;
            }
            thread::yield_now();
        }
        let batch = received.expect("no successful handoff within the test's yield budget");
        assert_eq!(batch.len(), 2);
        assert_eq!(batch.get(0).map(String::as_str), Some("header"));
        assert_eq!(batch.get(1).map(String::as_str), Some("payload"));
        assert_eq!(batch.get(2), None);
    });
}

#[test]
fn send_but_not_sync_payload_can_cross_the_handoff() {
    let cell = OneShot::<Cell<u32>>::new();
    thread::scope(|s| {
        s.spawn(|| {
            cell.try_publish(Cell::new(42)).unwrap();
        });
        let mut answer = None;
        for _ in 0..2048 {
            if let Some(value) = cell.try_take() {
                answer = Some(value);
                break;
            }
            thread::yield_now();
        }
        assert_eq!(answer.unwrap().get(), 42);
    });
}

#[test]
fn audit_keeps_unique_tickets_and_a_consistent_live_snapshot() {
    let audit = Audit::default();
    let gate = Barrier::new(5);
    thread::scope(|s| {
        for _ in 0..4 {
            let (audit, gate) = (&audit, &gate);
            s.spawn(move || {
                gate.wait();
                for _ in 0..12 {
                    audit.record(3);
                    thread::yield_now();
                }
            });
        }
        gate.wait();
        for _ in 0..24 {
            let snap = audit.snapshot();
            assert_eq!(snap.completed, snap.entries.len());
            assert_eq!(
                snap.total_bytes,
                snap.entries.iter().map(|(_, bytes)| bytes).sum()
            );
            thread::yield_now();
        }
    });
    let snap = audit.snapshot();
    assert_eq!((snap.completed, snap.total_bytes), (48, 144));
    let mut ids: Vec<_> = snap.entries.iter().map(|(id, _)| *id).collect();
    ids.sort_unstable();
    assert_eq!(ids, (0..48).collect::<Vec<_>>());
}

#[test]
fn complete_handoff_preserves_payloads_and_accounts_for_work() {
    let audit = Audit::default();
    let cell = OneShot::new();
    thread::scope(|s| {
        s.spawn(|| {
            let mut batch = Batch::<String, 4>::new();
            for text in ["aa", "bbb", "cccc"] {
                batch.push(text.to_string()).unwrap();
            }
            cell.try_publish(batch)
                .ok()
                .expect("only producer must win");
        });
        let mut received = None;
        for _ in 0..2048 {
            if let Some(batch) = cell.try_take() {
                received = Some(batch);
                break;
            }
            thread::yield_now();
        }
        let mut batch = received.expect("publication did not complete");
        while let Some(value) = batch.pop() {
            audit.record(value.len());
        }
        assert!(cell.try_take().is_none());
    });
    let snap = audit.snapshot();
    assert_eq!((snap.completed, snap.total_bytes), (3, 9));
}
