fn main() {
    // TASK_BEGIN demonstrate
    let notifications = Arc::new(AtomicUsize::new(0));
    let waker = make_waker(notifications.clone());
    println!("owners after create: {}", Arc::strong_count(&notifications));
    drop(waker);
    assert_eq!(Arc::strong_count(&notifications), 1);
    println!("owners after drop: 1");
    // TASK_END demonstrate
}

use std::mem::ManuallyDrop;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::task::{RawWaker, RawWakerVTable, Waker};
// Safety invariant: each raw waker owns exactly one Arc<AtomicUsize> strong reference.
// Data comes only from Arc::into_raw below; the vtable always uses that same type.
// Arc and AtomicUsize make these callbacks thread-safe. No borrowed callback consumes an owner.
fn into_raw(owner: Arc<AtomicUsize>) -> RawWaker {
    RawWaker::new(Arc::into_raw(owner).cast(), &VTABLE)
}
fn make_waker(owner: Arc<AtomicUsize>) -> Waker {
    // SAFETY: into_raw and VTABLE uphold the ownership and thread-safety invariant above.
    unsafe { Waker::from_raw(into_raw(owner)) }
}
unsafe fn clone_raw(data: *const ()) -> RawWaker {
    // SAFETY: data is the live Arc pointer associated with this vtable; do not consume it.
    let owner = ManuallyDrop::new(unsafe { Arc::from_raw(data.cast::<AtomicUsize>()) });
    into_raw(Arc::clone(&owner))
}
unsafe fn wake_raw(data: *const ()) {
    // SAFETY: the consuming wake transfers this raw owner to this callback exactly once.
    let owner = unsafe { Arc::from_raw(data.cast::<AtomicUsize>()) };
    owner.fetch_add(1, Ordering::Relaxed);
}
unsafe fn wake_ref_raw(data: *const ()) {
    // SAFETY: borrow the raw owner; ManuallyDrop keeps the calling waker's owner alive.
    let owner = ManuallyDrop::new(unsafe { Arc::from_raw(data.cast::<AtomicUsize>()) });
    owner.fetch_add(1, Ordering::Relaxed);
}
unsafe fn drop_raw(data: *const ()) {
    // SAFETY: the drop operation transfers this raw owner here exactly once.
    drop(unsafe { Arc::from_raw(data.cast::<AtomicUsize>()) });
}
static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_raw, wake_raw, wake_ref_raw, drop_raw);
