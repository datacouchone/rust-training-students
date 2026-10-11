//! Lab 6 — lock-free atomics, memory ordering, and a spinlock.
//!
//! Three pieces:
//!
//! 1. [`ConnectionTracker`] — independent lock-free counters.
//! 2. [`PublishSlot`] — the classic Acquire/Release message-passing example
//!    (a `ready` flag gating a data slot).
//! 3. [`SpinLock`] — a mutex built from a `bool` + `compare_exchange`.
//!
//! Every atomic operation below is written with `Ordering::SeqCst` as a
//! deliberately-conservative placeholder, marked with a `// TODO: can this
//! be weaker? justify` comment. `SeqCst` is always *correct* (it's the
//! strongest ordering), but it is rarely the *cheapest* correct choice, and
//! picking the weakest-correct ordering for each operation is the actual
//! point of this lab. For each TODO, work out: what happens-before edge (if
//! any) does this operation need to establish, and with which other
//! operation does it need to pair? Write your answer as a comment replacing
//! the TODO — you'll need that reasoning to defend the choice.
//!
//! `SpinLock::lock` itself is left as `todo!()`: that's the hands-on
//! exercise — implement the CAS spin loop. The `// SAFETY:` comments near
//! the guard's `Deref`/`DerefMut` are scaffolds: fill in *why* each unsafe
//! block is sound (what does holding a `SpinLockGuard` prove?).
//!
//! Background reading: Mara Bos, *Rust Atomics and Locks*, ch. 2–4; the
//! Rustonomicon's atomics section.

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// ============================================================================
// ConnectionTracker
// ============================================================================

/// A point-in-time snapshot of a [`ConnectionTracker`]'s counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    pub connections: u64,
    pub total_bytes: u64,
}

/// Lock-free counters for a fleet of connections: how many were registered,
/// and how many bytes they've moved in total.
pub struct ConnectionTracker {
    next_id: AtomicU64,
    connections: AtomicU64,
    total_bytes: AtomicU64,
}

impl ConnectionTracker {
    pub const fn new() -> Self {
        Self {
            next_id: AtomicU64::new(0),
            connections: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
        }
    }

    /// Registers a new connection and returns its unique id.
    pub fn register(&self) -> u64 {
        self.connections.fetch_add(1, Ordering::SeqCst);
        // TODO: can this be weaker? justify.
        self.next_id.fetch_add(1, Ordering::SeqCst)
        // TODO: can this be weaker? justify.
        // hint: does the *uniqueness* of the returned id depend on any
        // other thread being able to see some *other* memory location
        // through this counter, or only on `next_id` itself never handing
        // out the same value twice?
    }

    /// Records that `n` more bytes moved on some connection.
    pub fn record_bytes(&self, n: u64) {
        self.total_bytes.fetch_add(n, Ordering::SeqCst);
        // TODO: can this be weaker? justify.
    }

    /// Reads the current counters.
    pub fn snapshot(&self) -> Stats {
        Stats {
            connections: self.connections.load(Ordering::SeqCst),
            // TODO: can this be weaker? justify.
            total_bytes: self.total_bytes.load(Ordering::SeqCst),
            // TODO: can this be weaker? justify.
        }
    }
}

impl Default for ConnectionTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PublishSlot — Acquire/Release message passing
// ============================================================================

/// A single-shot, single-writer "publish once, read many" cell: the classic
/// Acquire/Release message-passing example (`ready: AtomicBool` gating a
/// `data` slot).
///
/// `publish` may be called at most once per instance. After it returns,
/// any thread may call `read`/`try_read` to obtain a shared reference to
/// the value, however many times, from however many threads.
pub struct PublishSlot<T> {
    data: UnsafeCell<MaybeUninit<T>>,
    /// Guards *who* gets to write `data` — exactly one winner.
    claimed: AtomicBool,
    /// The Acquire/Release synchronization point.
    ready: AtomicBool,
}

// SAFETY (Send): all fields are automatically `Send` when `T: Send`, so
// `PublishSlot<T>` derives `Send` without a manual impl.
//
// SAFETY (Sync): `read`/`try_read` can hand out `&T` to multiple threads
// *concurrently* (unlike `SpinLock`, which only ever exposes `&mut T` to
// one thread at a time). Soundly sharing `&T` across threads is what
// `Sync` means, so — unlike `SpinLock`'s `T: Send`-only bound — this type
// needs `T: Sync` too. `T: Send` is still required because the value can
// be constructed on one thread and dropped on another.
unsafe impl<T: Send + Sync> Sync for PublishSlot<T> {}

impl<T> PublishSlot<T> {
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            claimed: AtomicBool::new(false),
            ready: AtomicBool::new(false),
        }
    }

    /// Publishes `value`, making it visible to subsequent `read`/`try_read`
    /// calls on any thread.
    ///
    /// # Panics
    /// Panics if called more than once on the same `PublishSlot`.
    pub fn publish(&self, value: T) {
        let won = self
            .claimed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        // TODO: can either side of this CAS be weaker? justify. hint: this
        // CAS only decides *who* gets to write `data` among concurrent
        // `publish` callers — does picking a unique winner need any
        // cross-thread visibility guarantee for *other* memory, given that
        // `ready` (below) is what actually publishes `data`?
        assert!(won, "PublishSlot::publish called more than once");

        // SAFETY: TODO — justify why this write is sound. What does
        // winning the CAS above guarantee about concurrent access to
        // `data` at this point?
        unsafe {
            (*self.data.get()).write(value);
        }

        self.ready.store(true, Ordering::SeqCst);
        // TODO: can this be weaker? justify. hint: this store and the load
        // in `try_read` are a pair — what needs to be true of *this* store
        // relative to the `data.write` above, so that a reader who
        // observes `ready == true` can safely read `data`?
    }

    /// Non-blocking read: `None` until a value has been published.
    pub fn try_read(&self) -> Option<&T> {
        if self.ready.load(Ordering::SeqCst) {
            // TODO: can this load be weaker? justify. hint: this is the
            // other half of the pair with `publish`'s store into `ready` —
            // what ordering does it need so that reading `data` below,
            // once this observes `true`, is not a data race?

            // SAFETY: TODO — justify why it's sound to treat `data` as
            // initialized here, given the load above returned `true`.
            Some(unsafe { (*self.data.get()).assume_init_ref() })
        } else {
            None
        }
    }

    /// Blocking read: spins until a value has been published, then returns
    /// a reference to it.
    pub fn read(&self) -> &T {
        loop {
            if let Some(value) = self.try_read() {
                return value;
            }
            std::hint::spin_loop();
        }
    }
}

impl<T> Default for PublishSlot<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for PublishSlot<T> {
    fn drop(&mut self) {
        // `&mut self` proves exclusive access, so a plain (non-atomic)
        // `get_mut` read is enough here — no ordering to choose.
        if *self.ready.get_mut() {
            // SAFETY: `ready` observed `true` under `&mut self` (exclusive
            // access) means a value was published and fully initialized.
            unsafe {
                self.data.get_mut().assume_init_drop();
            }
        }
    }
}

// ============================================================================
// SpinLock — Acquire/Release mutual exclusion
// ============================================================================

/// A minimal spinlock-based mutex, built from `AtomicBool::compare_exchange`.
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: `UnsafeCell<T>` makes `SpinLock<T>` `!Sync` by default — the
// compiler has no way to see that `locked` mediates every access to `data`.
// We assert it by hand: `lock()`/`try_lock()` guarantee at most one
// `SpinLockGuard` (and hence at most one `&mut T`) exists at a time, so
// sharing `&SpinLock<T>` across threads is sound.
//
// `T: Send` (not `T: Sync`) is the right bound, matching
// `std::sync::Mutex<T>`: only one thread ever touches the `T` at a time,
// but ownership of that access hops between threads each time the lock
// changes hands. One deliberate divergence: std's `MutexGuard` is `!Send` (OS
// mutexes require unlock on the owning thread); this atomic `SpinLockGuard`
// has no such thread-affinity requirement, so we don't opt out of `Send` —
// releasing via a `Release` store is thread-agnostic.
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Acquires the lock, spinning until it is free.
    ///
    /// TODO: implement this. Spin on `self.locked`, using
    /// `compare_exchange`/`compare_exchange_weak` to attempt the
    /// `false -> true` transition, until you win. Consider a "test, then
    /// test-and-set" loop: spin on a plain `load` while the lock looks
    /// held, and only attempt the (more expensive) CAS once it looks free.
    ///
    /// Ordering: choose the orderings for the CAS's success and failure
    /// cases. Start from `Ordering::SeqCst` for both if you want, then ask:
    /// which one pairs with `SpinLockGuard::drop`'s unlock store, and what
    /// does that pairing need to guarantee about the *previous* lock
    /// holder's writes? Does the failure case (you didn't get the lock)
    /// need to synchronize with anything at all?
    #[must_use]
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        todo!(
            "spin on compare_exchange (or compare_exchange_weak) until `locked` flips false -> true, then return SpinLockGuard {{ lock: self }}"
        )
    }

    /// Attempts to acquire the lock without spinning; `None` if contended.
    #[must_use]
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        self.locked
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            // TODO: can either side of this CAS be weaker? justify —
            // treat success and failure as two separate questions.
            .ok()
            .map(|_| SpinLockGuard { lock: self })
    }
}

/// RAII guard returned by [`SpinLock::lock`]/[`SpinLock::try_lock`]. Unlocks
/// on `Drop`.
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: TODO — fill in the reasoning. What does merely *holding*
        // a `SpinLockGuard` prove about who else can be accessing `data`
        // right now?
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: TODO — same question as `deref`, but for exclusive
        // (`&mut`) access. What does the `&mut self` parameter add here?
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::SeqCst);
        // TODO: can this be weaker? justify. hint: this store is the other
        // half of a pair with the successful CAS in `lock`/`try_lock` —
        // what needs to be true of this store relative to everything this
        // thread did through the guard, so the *next* lock holder sees it?
    }
}

// ============================================================================
// Tests — these define "done" for this lab. `SpinLock::lock` is `todo!()`
// above, so `test_spinlock_mutual_exclusion` and `test_try_lock_respects_
// held_lock` will panic until you implement it; the rest should already
// pass against this starter as shipped.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::thread;

    const THREADS: usize = 8;
    const ITERS: usize = 1000;

    /// `register()` must hand out globally unique ids under real concurrency.
    /// Assertion is deterministic (set of ids has no duplicates and the
    /// right cardinality) regardless of how threads actually interleave.
    #[test]
    fn test_concurrent_register_unique_ids() {
        let tracker = Arc::new(ConnectionTracker::new());
        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                let tracker = Arc::clone(&tracker);
                thread::spawn(move || {
                    let mut ids = Vec::with_capacity(ITERS);
                    for _ in 0..ITERS {
                        ids.push(tracker.register());
                    }
                    ids
                })
            })
            .collect();

        let mut seen = HashSet::with_capacity(THREADS * ITERS);
        for h in handles {
            for id in h.join().expect("worker thread panicked") {
                assert!(seen.insert(id), "duplicate connection id {id}");
            }
        }
        assert_eq!(seen.len(), THREADS * ITERS);
        assert_eq!(
            tracker.snapshot().connections,
            (THREADS * ITERS) as u64,
            "connection counter must equal total register() calls"
        );
    }

    /// `total_bytes` and `connections` must exactly match the sums a
    /// caller can independently compute, after joining all writer threads
    /// (join is itself a happens-before edge, so this read is race-free).
    #[test]
    fn test_stats_sum_consistent() {
        const BYTES_PER_CALL: u64 = 37;

        let tracker = Arc::new(ConnectionTracker::new());
        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                let tracker = Arc::clone(&tracker);
                thread::spawn(move || {
                    for _ in 0..ITERS {
                        tracker.register();
                        tracker.record_bytes(BYTES_PER_CALL);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("worker thread panicked");
        }

        let stats = tracker.snapshot();
        assert_eq!(stats.connections, (THREADS * ITERS) as u64);
        assert_eq!(stats.total_bytes, (THREADS * ITERS) as u64 * BYTES_PER_CALL);
    }

    /// Acquire/Release message passing: a writer thread publishes a
    /// non-trivial heap value, a reader thread spins until it's visible and
    /// reads it back. Repeated many times (loom is not used here — plain
    /// threads + repetition per the lab's constraints) to build empirical
    /// confidence; correctness in each iteration is asserted exactly
    /// (payload equality), never inferred from timing.
    #[test]
    fn test_acquire_release_publish() {
        const REPETITIONS: usize = 3000;

        for i in 0..REPETITIONS {
            let slot = Arc::new(PublishSlot::new());
            let payload: Vec<u32> = (0..64).map(|x| x + i as u32).collect();
            let expected = payload.clone();

            let writer = {
                let slot = Arc::clone(&slot);
                thread::spawn(move || slot.publish(payload))
            };
            let reader = {
                let slot = Arc::clone(&slot);
                thread::spawn(move || slot.read().clone())
            };

            writer.join().expect("writer thread panicked");
            let observed = reader.join().expect("reader thread panicked");
            assert_eq!(
                observed, expected,
                "publish/read payload mismatch on iteration {i}"
            );
        }
    }

    /// The guard must provide real mutual exclusion: many threads doing a
    /// non-atomic read-modify-write inside the critical section must
    /// produce the exact expected final count, deterministically, no
    /// matter how the spinning interleaves.
    #[test]
    fn test_spinlock_mutual_exclusion() {
        let lock = Arc::new(SpinLock::new(0u64));
        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                let lock = Arc::clone(&lock);
                thread::spawn(move || {
                    for _ in 0..ITERS {
                        let mut guard = lock.lock();
                        *guard += 1;
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("worker thread panicked");
        }

        assert_eq!(*lock.lock(), (THREADS * ITERS) as u64);
    }

    /// `try_lock` must fail while the lock is held and succeed once it's
    /// free — checked without any sleeps by using the held guard itself as
    /// the synchronization point.
    #[test]
    fn test_try_lock_respects_held_lock() {
        let lock = SpinLock::new(5);
        let guard = lock.lock();
        assert!(lock.try_lock().is_none(), "try_lock must fail while held");
        drop(guard);
        let guard = lock.try_lock().expect("try_lock must succeed once free");
        assert_eq!(*guard, 5);
    }
}
