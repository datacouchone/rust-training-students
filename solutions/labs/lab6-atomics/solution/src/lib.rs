//! Lab 6 — lock-free atomics, memory ordering, and a spinlock.
//!
//! Three pieces, each picked to teach a different ordering lesson:
//!
//! 1. [`ConnectionTracker`] — independent counters. Nothing here needs a
//!    happens-before edge to *other* memory, so every operation is
//!    `Ordering::Relaxed`. The lesson: `Relaxed` fetch-add/CAS still gives
//!    you a single, total, per-object modification order — that's enough
//!    for uniqueness and for exact sums, without any inter-thread
//!    visibility guarantee.
//! 2. [`PublishSlot`] — the classic Acquire/Release message-passing example
//!    (a `ready` flag gating a data slot). This is where `Relaxed` would be
//!    a genuine bug: the payload write must *happen-before* the reader's
//!    access, and only Release/Acquire (or stronger) provides that edge.
//! 3. [`SpinLock`] — a mutex built from a `bool` + `compare_exchange`. Same
//!    Acquire/Release pairing as `PublishSlot`, but framed as "protect
//!    everything written during the critical section" instead of "protect
//!    one publish".
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
///
/// Every field is an independent `AtomicU64`. None of them ever gate access
/// to some *other* piece of shared memory (there's no payload being
/// "published" alongside a counter bump), so none of them need Acquire or
/// Release: `Relaxed` is the weakest — and here, the *correct* — choice.
///
/// Why `Relaxed` is still correct (not just fast): the C++/Rust memory model
/// guarantees each individual atomic object has a single, total
/// modification order that *every* thread agrees on, independent of the
/// ordering used to access it. A `fetch_add` always reads-modifies-writes
/// the value that is "next" in that total order — two concurrent
/// `fetch_add`s can never both observe the same prior value, so they can
/// never return the same result. That's exactly what uniqueness needs, and
/// exactly what an exact running sum needs (every increment is counted
/// exactly once, in *some* order — which order doesn't matter for a sum).
/// What `Relaxed` does *not* give you is a way to know that a bump you
/// observed implies anything about *other* memory — but `ConnectionTracker`
/// never makes that promise, so it never needs to pay for it.
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
    ///
    /// Ordering: `Relaxed`. See the type-level docs — uniqueness comes from
    /// `fetch_add`'s atomicity within `next_id`'s single total modification
    /// order, not from any cross-thread visibility this call would need to
    /// establish for other memory.
    pub fn register(&self) -> u64 {
        self.connections.fetch_add(1, Ordering::Relaxed);
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Records that `n` more bytes moved on some connection.
    ///
    /// Ordering: `Relaxed`, same reasoning — an independent running total,
    /// not paired with any other write that needs to become visible.
    pub fn record_bytes(&self, n: u64) {
        self.total_bytes.fetch_add(n, Ordering::Relaxed);
    }

    /// Reads the current counters.
    ///
    /// Ordering: `Relaxed`. Note this snapshot is *not* atomic as a whole —
    /// `connections` and `total_bytes` are read independently and could be
    /// torn relative to each other under concurrent writers. That's fine
    /// for this type (each counter is individually consistent, which is all
    /// it promises). A caller that needs the snapshot to happen-after a
    /// specific set of writer threads gets that from `thread::join`
    /// (joining a thread is itself a happens-before edge), not from the
    /// ordering used here.
    pub fn snapshot(&self) -> Stats {
        Stats {
            connections: self.connections.load(Ordering::Relaxed),
            total_bytes: self.total_bytes.load(Ordering::Relaxed),
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
///
/// This is deliberately the simple "publish once" shape (not a full
/// multi-writer seqlock with a sequence counter and retry-on-torn-read) —
/// the point is the Acquire/Release *pairing*, not seqlock's retry
/// machinery.
pub struct PublishSlot<T> {
    data: UnsafeCell<MaybeUninit<T>>,
    /// Guards *who* gets to write `data` — exactly one winner, enforced by
    /// `claimed`'s single total modification order (same trick as
    /// `ConnectionTracker::register`). This flag is not what makes `data`
    /// visible to readers; `ready` does that.
    claimed: AtomicBool,
    /// The Acquire/Release synchronization point. A `Release` store here
    /// happens after the payload write; an `Acquire` load that observes
    /// `true` happens-before any subsequent read of `data`.
    ready: AtomicBool,
}

// SAFETY (Send): all fields (`UnsafeCell<MaybeUninit<T>>`, two `AtomicBool`s)
// are automatically `Send` when `T: Send`, so `PublishSlot<T>` derives `Send`
// without a manual impl.
//
// SAFETY (Sync): unlike `SpinLock<T>` (which only ever exposes `&mut T` to
// one thread at a time), `PublishSlot::read`/`try_read` can hand out `&T` to
// *multiple threads concurrently* once published — any number of readers
// can call `read()` in parallel and all get a live `&T` to the same value at
// the same time. Soundly sharing `&T` across threads is *precisely* what
// `Sync` means (`T: Sync` iff `&T: Send`), so unlike `SpinLock`'s
// `T: Send`-only bound, `PublishSlot` needs `T: Sync` too. `T: Send` is
// still separately required because the value can be constructed on one
// thread (the publisher) and dropped on another (whichever thread drops the
// last handle to the slot) — an ownership transfer across threads.
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
        // Ordering: Relaxed on both outcomes. This CAS only needs to pick a
        // unique winner among concurrent `publish` callers (mutual
        // exclusion over *this* atomic's total modification order); it does
        // not need to establish visibility of `data` for anyone — that is
        // entirely the job of the `ready` Release store below. There is
        // also no prior writer's data this call needs to observe (we are
        // the writer), so Acquire on success would buy nothing.
        let won = self
            .claimed
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok();
        assert!(won, "PublishSlot::publish called more than once");

        // SAFETY: the CAS above succeeded for exactly one call (guaranteed
        // by `claimed`'s single total modification order — only one thread
        // can ever observe the `false -> true` transition), so we have
        // exclusive access to `data` here. No reader can be observing it
        // yet either: `ready` is still `false` (we haven't stored to it),
        // and any reader that later sees `ready == true` will do so only
        // *after* the Release store below, i.e. after this write completes.
        unsafe {
            (*self.data.get()).write(value);
        }

        // Ordering: Release. This is the write half of the happens-before
        // edge this type exists to demonstrate. Every write sequenced
        // before this store — in particular, the `data.write(value)` above
        // — happens-before any load that reads `true` from this store with
        // `Acquire` (or stronger). `Relaxed` here would be a real bug: the
        // compiler/hardware would be free to make `ready` visible to
        // another core before `data` is, letting a reader observe
        // `ready == true` and then read uninitialized/torn memory.
        self.ready.store(true, Ordering::Release);
    }

    /// Non-blocking read: `None` until a value has been published.
    pub fn try_read(&self) -> Option<&T> {
        // Ordering: Acquire. Pairs with the `Release` store in `publish`.
        // If this load observes `true`, then by the definition of
        // Release/Acquire synchronization, the `Release` store
        // happens-before this `Acquire` load — and everything sequenced
        // before that store (the payload write) therefore happens-before
        // everything sequenced after this load (the read below). That's
        // exactly the guarantee needed to read `data` without a race.
        // `Relaxed` here would break the pairing: the load could observe
        // `true` (the *value* is still atomic) without any ordering
        // guarantee about `data`, so the read below would be a data race —
        // undefined behavior — even though it would "usually" work on
        // strongly-ordered hardware.
        if self.ready.load(Ordering::Acquire) {
            // SAFETY: `ready == true`, observed via the Acquire load above,
            // proves (via the happens-before edge described there) that the
            // write in `publish` has completed and is visible here. `data`
            // is therefore initialized and safe to read as `&T`. It stays
            // safe to read for the lifetime of `&self` because nothing
            // ever mutates `data` again after `publish` (single-shot).
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
        // `&mut self` proves exclusive access (no other thread can be
        // reading or writing concurrently), so plain `get_mut` — no atomic
        // operation needed — is enough to check whether a value was ever
        // published.
        if *self.ready.get_mut() {
            // SAFETY: `ready` was observed `true` under `&mut self`
            // (exclusive access, so this can't be racing a concurrent
            // `publish`), which only happens after `publish` fully
            // initialized `data`. It is therefore sound to drop the `T`
            // stored inside.
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
///
/// Same Acquire/Release pairing as [`PublishSlot`], but framed around a
/// critical section instead of a single publish: `lock()`'s successful CAS
/// is the Acquire half, `Drop` for the guard is the Release half. Everything
/// the previous lock holder wrote during their critical section (sequenced
/// before their Release unlock) happens-before everything the next lock
/// holder does after their Acquire lock — which is exactly mutual exclusion
/// *plus* visibility of the previous holder's writes.
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
// `T: Send` (not `T: Sync`) is the right bound — and it matches
// `std::sync::Mutex<T>` exactly. Only one thread ever touches the `T` at a
// time (never concurrent `&T` from two threads simultaneously, unlike
// `PublishSlot`), but ownership of that access hops from thread to thread
// every time the lock changes hands — the same requirement as sending `T`
// across threads. `Sync` would be a stronger bound than this type needs.
// One deliberate divergence: std's `MutexGuard` is `!Send` (OS mutexes
// require unlock on the owning thread); this atomic `SpinLockGuard` has no
// such thread-affinity requirement, so we don't opt out of `Send` — releasing
// via a `Release` store is thread-agnostic.
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Acquires the lock, spinning until it is free.
    #[must_use]
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        // Test-and-test-and-set (TTAS): a bare `Relaxed` load doesn't
        // require exclusive cache-line ownership the way a CAS does, so
        // under contention it's much cheaper to spin on a load and only
        // attempt the (more expensive) CAS once the lock *looks* free.
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }
        // Ordering used above: Acquire on success, Relaxed on failure.
        //   - Success (Acquire): this is the read half of the pairing with
        //     `SpinLockGuard::drop`'s Release store. It must happen-before
        //     everything we do with the guard, so that the previous lock
        //     holder's writes (sequenced-before their Release unlock) are
        //     visible to us. `Relaxed` would let us observe `locked` flip
        //     to `false -> true` (the CAS is still atomic) without any
        //     guarantee that the previous critical section's writes are
        //     visible — a data race on `data`.
        //   - Failure (Relaxed): a failed CAS reads no data we depend on —
        //     we're about to spin and retry, so there's nothing to
        //     synchronize with yet. Any ordering here is only paid for, not
        //     used; `compare_exchange_weak` also permits spurious failure,
        //     which the retry loop already has to handle regardless of
        //     ordering.
        SpinLockGuard { lock: self }
    }

    /// Attempts to acquire the lock without spinning; `None` if contended.
    #[must_use]
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        // Same ordering reasoning as `lock`'s CAS: Acquire on success pairs
        // with the guard's Release unlock; Relaxed on failure since a
        // failed attempt observes nothing we act on.
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
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
        // SAFETY: holding a `SpinLockGuard` is proof that this thread won
        // the CAS in `lock`/`try_lock` and no matching `Drop` (unlock) has
        // run yet, so no other `SpinLockGuard` for this `SpinLock` can
        // exist concurrently — this is the only live access to `data`.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: same exclusivity argument as `deref`; `&mut self` here
        // additionally rules out an alias through this guard itself.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        // Ordering: Release. This is the write half of the pairing with
        // the Acquire CAS in `lock`/`try_lock`. Everything this thread did
        // through the guard (sequenced-before this store) happens-before
        // the next successful Acquire CAS on `locked` — which is exactly
        // the visibility guarantee the next lock holder needs.
        // `Relaxed` here would be a genuine bug: the next `lock()` could
        // observe `locked == false` (the flip is still atomic) without any
        // guarantee that our critical-section writes are visible yet.
        self.lock.locked.store(false, Ordering::Release);
    }
}

// ============================================================================
// Tests
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
