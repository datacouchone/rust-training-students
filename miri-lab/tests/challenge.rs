use miri_lab::{checked_read, increment_atomic, increment_locked, initialize_slot};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[test]
fn a_read_boundaries() {
    assert_eq!(checked_read(&[], 0), None);
    assert_eq!(checked_read(&[11, 22, 33], 0), Some(11));
    assert_eq!(checked_read(&[11, 22, 33], 2), Some(33));
    assert_eq!(checked_read(&[11, 22, 33], 3), None);
    assert_eq!(checked_read(&[11], usize::MAX), None);
}

#[test]
fn b_initialize_before_reading() {
    for value in [0, 1, 42, u32::MAX] {
        assert_eq!(initialize_slot(value), value);
    }
}

#[test]
fn c_locked_counter_keeps_all_updates() {
    let count = Mutex::new(0);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let count = &count;
            scope.spawn(move || {
                for _ in 0..16 {
                    increment_locked(count);
                }
            });
        }
    });
    assert_eq!(*count.lock().unwrap(), 64);
}

#[test]
fn d_atomic_counter_keeps_all_updates() {
    let count = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let count = &count;
            scope.spawn(move || {
                for _ in 0..16 {
                    increment_atomic(count);
                }
            });
        }
    });
    assert_eq!(count.load(Ordering::Relaxed), 64);
}
