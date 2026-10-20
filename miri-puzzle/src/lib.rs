//! Capstone: own a batch, publish it once, transfer it once, account for work.
//! Keep the representations and public signatures. Implement the eight TODOs.
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::{
    Mutex,
    atomic::{AtomicU8, AtomicUsize, Ordering},
};

/// Exactly slots [0, len) contain live T values. Remaining slots are uninitialized.
/// The Box owns the allocation; neither a raw address nor spare capacity owns T.
/// N = 0 and zero-sized T are supported. No Clone or Copy bound is permitted.
///
/// A borrowed element must not outlive its batch:
/// ```compile_fail
/// use miri_puzzle::Batch;
/// let view;
/// {
///     let mut batch = Batch::<String, 2>::new();
///     batch.push(String::from("owned")).unwrap();
///     view = batch.get(0).unwrap();
/// }
/// println!("{view}");
/// ```
/// Nor can it remain used across a mutation:
/// ```compile_fail
/// use miri_puzzle::Batch;
/// let mut batch = Batch::<String, 2>::new();
/// batch.push(String::from("owned")).unwrap();
/// let view = batch.get(0).unwrap();
/// let removed = batch.pop();
/// println!("{view} {removed:?}");
/// ```
pub struct Batch<T, const N: usize> {
    slots: Box<[MaybeUninit<T>; N]>,
    len: usize,
}

impl<T, const N: usize> Batch<T, N> {
    pub fn new() -> Self {
        Self {
            slots: Box::new(std::array::from_fn(|_| MaybeUninit::uninit())),
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn capacity(&self) -> usize {
        N
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Return the original value unchanged on full capacity; do not destroy it.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        todo!("A: check capacity, initialize one slot, then commit length")
    }

    /// Transfer the final initialized T out. Empty returns None.
    pub fn pop(&mut self) -> Option<T> {
        todo!("B: transfer ownership without leaving a second live owner")
    }

    /// Borrow an initialized element, never spare capacity. Huge indices are valid input.
    pub fn get(&self, index: usize) -> Option<&T> {
        todo!("C: establish bounds before forming or reading an element pointer")
    }
}
impl<T, const N: usize> Default for Batch<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const N: usize> Drop for Batch<T, N> {
    fn drop(&mut self) {
        if self.len != 0 {
            todo!("D: destroy every remaining live value exactly once, not every slot")
        }
    }
}

// One lifecycle only: EMPTY -> WRITING -> READY -> TAKEN. No reset or reuse.
const EMPTY: u8 = 0;
const WRITING: u8 = 1;
const READY: u8 = 2;
const TAKEN: u8 = 3;

/// Multiple publishers may compete; at most one succeeds. At most one taker wins.
/// A failed try_take is a nonblocking observation, not a promise of permanent absence.
/// No borrowed reference to the payload escapes this type.
///
/// A non-Send payload must not become transferable through this wrapper:
/// ```compile_fail
/// use miri_puzzle::OneShot;
/// use std::{rc::Rc, thread};
/// let cell = OneShot::<Rc<u32>>::new();
/// thread::scope(|s| { s.spawn(|| { let _ = cell.try_take(); }); });
/// ```
pub struct OneShot<T> {
    slot: UnsafeCell<MaybeUninit<T>>,
    state: AtomicU8,
}

// AUDIT REQUIRED: justify this promise using exclusive writer/taker claims,
// publication synchronization, T: Send, and the absence of shared payload views.
// UnsafeCell itself does not synchronize any access.
unsafe impl<T: Send> Sync for OneShot<T> {}

impl<T> OneShot<T> {
    pub fn new() -> Self {
        Self {
            slot: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicU8::new(EMPTY),
        }
    }

    /// Claim before touching slot. On failure return this call's original value.
    /// Publish only after initialization. Do not run user callbacks while WRITING.
    pub fn try_publish(&self, value: T) -> Result<(), T> {
        todo!("E: reserve the only writer, initialize, then publish")
    }

    /// Only the winning taker may read/move the payload. Never clone or reset it.
    pub fn try_take(&self) -> Option<T> {
        todo!("F: observe publication and claim the only transfer")
    }
}
impl<T> Default for OneShot<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Drop for OneShot<T> {
    fn drop(&mut self) {
        if *self.state.get_mut() == READY {
            todo!("G: destroy an initialized payload only if ownership was not transferred")
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuditSnapshot {
    pub completed: usize,
    pub total_bytes: usize,
    pub entries: Vec<(usize, usize)>, // (ticket, bytes)
}

/// next_ticket allocates unique IDs; it does NOT publish the journal's contents.
/// Each snapshot must have completed == entries.len() and the matching byte sum.
/// Workloads in this exercise do not overflow usize; destructors do not panic.
#[derive(Default)]
pub struct Audit {
    next_ticket: AtomicUsize,
    journal: Mutex<AuditSnapshot>,
}
impl Audit {
    /// One atomic ticket allocation; one critical section for the whole journal update.
    pub fn record(&self, bytes: usize) -> usize {
        todo!("H: prevent both lost IDs and internally inconsistent accounting")
    }
    pub fn snapshot(&self) -> AuditSnapshot {
        self.journal.lock().expect("journal poisoned").clone()
    }
}
