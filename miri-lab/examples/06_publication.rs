use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

struct Mailbox {
    payload: UnsafeCell<u32>,
    ready: AtomicBool,
}
// Intended invariant: one writer; reader accesses payload only after publication.
// BUG: the current flag orderings do not establish that publication.
unsafe impl Sync for Mailbox {}

fn main() {
    assert!(
        cfg!(miri),
        "Intentional invalid example: run with cargo miri run --example 06_publication"
    );
    let mailbox = Mailbox {
        payload: UnsafeCell::new(0),
        ready: AtomicBool::new(false),
    };
    std::thread::scope(|scope| {
        let mailbox = &mailbox; // Share the complete wrapper, not individual fields.
        scope.spawn(move || {
            // One writer, one write. The reader must be synchronized with it.
            unsafe {
                *mailbox.payload.get() = 42;
            }
            mailbox.ready.store(true, Ordering::Relaxed);
        });
        scope.spawn(move || {
            while !mailbox.ready.load(Ordering::Relaxed) {
                std::thread::yield_now();
            }
            // Seeing a relaxed flag is not permission to read non-atomic data.
            let value = unsafe { *mailbox.payload.get() };
            assert_eq!(value, 42);
        });
    });
}
