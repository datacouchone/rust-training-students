//! Repair exercises. Run baseline first; challenge tests need these bodies.
use std::mem::MaybeUninit;
use std::sync::{Mutex, atomic::AtomicUsize};

/// Return a copied element if index is inside the initialized slice.
/// Empty input and index == len must return None. Do not use unsafe.
pub fn checked_read(values: &[u32], index: usize) -> Option<u32> {
    todo!("A: distinguish initialized elements from an address")
}

/// Store one value, then transfer that initialized value out of the slot.
/// Keep MaybeUninit; exactly one write must precede assume_init.
/// Explain the safety argument beside the unsafe operation.
pub fn initialize_slot(value: u32) -> u32 {
    let mut slot = MaybeUninit::<u32>::uninit();
    todo!("B: initialize before reading")
}

/// Add one under a single lock guard. Never split read and write across locks.
/// Inputs in this lab do not overflow and the mutex is not poisoned.
pub fn increment_locked(counter: &Mutex<usize>) {
    todo!("C: keep the whole read-modify-write under the lock")
}

/// Add one as a single atomic read-modify-write operation.
/// This counter publishes no other data; explain the chosen ordering.
pub fn increment_atomic(counter: &AtomicUsize) {
    todo!("D: one indivisible increment, not separate load/store")
}
