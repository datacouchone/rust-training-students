//! `MiniVec<T>`: a minimal, safe abstraction over a raw heap allocation.
//!
//! This is a teaching-sized re-implementation of the core of `std::vec::Vec`
//! (see the Rustonomicon's "Implementing Vec" chapter for the canonical
//! version). It exists to practice:
//!
//! - manual allocation with `std::alloc::{alloc, realloc, dealloc}` and
//!   `Layout`,
//! - the `NonNull<T>` + `PhantomData<T>` idiom for a raw-pointer-owning
//!   struct that still participates correctly in variance and drop-check
//!   (see below for what that means and why each half of the idiom earns
//!   its place), and
//! - writing `unsafe` blocks with a real, checkable safety argument for
//!   every one of them.
//!
//! ## `NonNull<T>` + `PhantomData<T>`: variance and drop-check, precisely
//!
//! **Variance.** `*mut T` is invariant, so `MiniVec<&'long str>` couldn't
//! stand in for `MiniVec<&'short str>` even though an owning collection
//! should allow it (the same way `Vec<T>` does). `NonNull<T>` is defined
//! via `*const T` internally, so it's covariant in `T` like `Vec<T>`
//! needs to be, and it buys the null-pointer niche for free:
//! `Option<MiniVec<T>>` is the same size as `MiniVec<T>` (`None` reuses
//! the all-zero-pointer bit pattern `NonNull` forbids), which a bare
//! `*mut T` field would not give us.
//!
//! **Drop-check.** As soon as a type has *any* manual, plain (non-`#[may_dangle]`)
//! `impl<T> Drop for MiniVec<T>`, rustc's drop checker (dropck) is already
//! conservative about it: it assumes the drop glue might do *anything* with
//! `T` — including reading through borrows `T` holds — so it forces every
//! `T` used in the vec to strictly outlive the vec's *drop point*, not just
//! its last ordinary use. That is true with or without `_marker:
//! PhantomData<T>` on the struct; the field changes nothing about it. This
//! errors with E0597/E0505 today, `PhantomData` or not — try it:
//!
//! ```rust,ignore
//! struct Loud<'a>(&'a str);
//! impl<'a> Drop for Loud<'a> {
//!     fn drop(&mut self) { println!("{}", self.0); } // reads through `'a`
//! }
//! let mut v: MiniVec<Loud> = MiniVec::new();
//! let s = String::from("short-lived");
//! v.push(Loud(&s));
//! drop(s);  // ERROR: `s` does not live long enough (E0597), and the
//!           // subsequent `drop(s)` move is also rejected (E0505) —
//!           // because `v`'s plain `Drop` impl means `v` is assumed to
//!           // (transitively) drop a `Loud<'_>`, which must still be
//!           // valid when `v` itself is dropped at the end of `main`.
//! ```
//!
//! (Delete `MiniVec`'s `Drop` impl entirely — no replacement, not even an
//! empty one — and the same program *does* compile, because then dropck
//! has nothing telling it `MiniVec` runs any destructor at all. That's a
//! deliberately different, worse program: it leaks the allocation instead
//! of freeing it. It is not what removing just the `PhantomData` field
//! does.)
//!
//! So if a plain `Drop` impl already makes dropck this strict on its own,
//! why keep `_marker: PhantomData<T>` at all? Two reasons, neither of them
//! "makes today's dropck stricter":
//!
//! 1. It declares logical ownership of `T` up front. That declaration is
//!    inert *right now*, but it becomes load-bearing the moment this impl
//!    reaches for the unstable `#[may_dangle]` attribute the way
//!    `std::vec::Vec` (via `RawVec`) actually does: `#[may_dangle]`
//!    *relaxes* dropck's default down to "this drop glue won't read
//!    through `T`'s borrows", and it is specifically the combination of
//!    `#[may_dangle]` *and* a `PhantomData<T>` field that lets the
//!    compiler soundly reinstate "but `T` must still be dropped/valid"
//!    just for the cases that need it, without over- or under-constraining
//!    borrows elsewhere in the struct. Neither half does that job alone.
//! 2. It's the documented Rustonomicon convention for any type that owns a
//!    `T` behind a raw pointer: it makes the ownership visible to a reader
//!    (and to future-you) instead of leaving it implicit in the `Drop`
//!    impl's body. That matters for refactoring safety — e.g. if a future
//!    change ever removes or weakens `MiniVec`'s `Drop` impl, a
//!    `PhantomData<T>` field is a visible signal (and, in variance/auto-trait
//!    terms, a real effect) that this struct still means to own a `T`,
//!    where a bare `NonNull<T>` alone would quietly stop conveying that.
//!
//! **Takeaway:** `PhantomData` here is about future-proofing and
//! `#[may_dangle]` soundness, not about making today's plain-`Drop` dropck
//! stricter — that strictness already comes from the `Drop` impl alone.

use std::alloc::{self, Layout};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr::{self, NonNull};

pub struct MiniVec<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T> MiniVec<T> {
    /// Creates an empty `MiniVec` without allocating.
    ///
    /// # Panics
    ///
    /// Panics if `T` is a zero-sized type. Supporting ZSTs correctly
    /// requires treating "pointer offset" and "capacity" specially
    /// throughout (a ZST allocation never actually needs memory), which is
    /// out of scope for this lab, so we reject them explicitly instead of
    /// silently doing the wrong thing.
    pub fn new() -> Self {
        assert_ne!(
            mem::size_of::<T>(),
            0,
            "MiniVec<T> does not support zero-sized types"
        );
        MiniVec {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
            _marker: PhantomData,
        }
    }

    /// Creates an empty `MiniVec` with room for at least `capacity`
    /// elements without reallocating.
    ///
    /// # Panics
    ///
    /// Panics if `T` is a zero-sized type, if the requested layout would
    /// overflow `isize`, or if the allocator fails to satisfy the request.
    pub fn with_capacity(capacity: usize) -> Self {
        assert_ne!(
            mem::size_of::<T>(),
            0,
            "MiniVec<T> does not support zero-sized types"
        );

        if capacity == 0 {
            return Self::new();
        }

        let layout = Layout::array::<T>(capacity).expect("capacity overflow");
        // `Layout::array` itself already enforces `layout.size() <=
        // isize::MAX`: it returns `Err` (turned into the `expect` panic
        // above) if the requested array size would overflow `isize` once
        // rounded up to `T`'s alignment, so no separate check is needed.

        // SAFETY: `layout` has non-zero size because `capacity > 0` (this
        // `if` branch) and `T` is non-zero-sized (asserted above), which is
        // `GlobalAlloc::alloc`'s only precondition beyond `layout` itself
        // being well-formed (guaranteed by `Layout::array`).
        let raw_ptr = unsafe { alloc::alloc(layout) };
        let ptr = match NonNull::new(raw_ptr.cast::<T>()) {
            Some(p) => p,
            None => alloc::handle_alloc_error(layout),
        };

        MiniVec {
            ptr,
            len: 0,
            cap: capacity,
            _marker: PhantomData,
        }
    }

    /// Grows the backing allocation: from `0` to `1` element, or by
    /// doubling otherwise. Called by `push` whenever `len == cap`.
    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = self.cap.checked_mul(2).expect("capacity overflow");
            let new_layout = Layout::array::<T>(new_cap).expect("capacity overflow");
            (new_cap, new_layout)
        };

        // `Layout::array` already enforces `new_layout.size() <=
        // isize::MAX` (see `with_capacity`), so no separate check is
        // needed here either.

        let raw_ptr = if self.cap == 0 {
            // SAFETY: `new_layout` describes a single, non-zero-sized `T`
            // (non-zero-sizedness was asserted once in `new`/
            // `with_capacity` and never changes), satisfying `alloc`'s only
            // precondition.
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            // SAFETY: `self.ptr` was returned by the global allocator with
            // exactly `old_layout` (either from `with_capacity` or a prior
            // `grow`, and `self.cap` has not changed since), so `old_layout`
            // matches the layout the allocation was made with, as
            // `realloc` requires. `new_layout.size()` is non-zero (it is
            // `size_of::<T>() * new_cap` with `new_cap >= 1` and `T`
            // non-zero-sized).
            unsafe {
                alloc::realloc(
                    self.ptr.as_ptr().cast::<u8>(),
                    old_layout,
                    new_layout.size(),
                )
            }
        };

        self.ptr = match NonNull::new(raw_ptr.cast::<T>()) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    /// Appends `value` to the end, growing the backing allocation first if
    /// there is no spare capacity.
    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }

        // SAFETY: the `grow` call above (a no-op if there was already
        // spare capacity) guarantees `self.len < self.cap`, so
        // `self.ptr.as_ptr().add(self.len)` is an in-bounds pointer to an
        // allocated-but-not-yet-initialized `T` slot within the current
        // allocation. `ptr::write` moves `value` into that slot without
        // running a destructor on whatever bits were already there, which
        // is required since that slot is not a live `T` yet.
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), value);
        }
        self.len += 1;
    }

    /// Removes and returns the last element, or `None` if the vector is
    /// empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;

        // SAFETY: before the decrement, indices `[0, old_len)` were live,
        // initialized `T` values; `self.len` now equals `old_len - 1`,
        // which is therefore still a live, initialized element. `ptr::read`
        // bit-copies it out; because we already shrank `self.len`, no
        // other `MiniVec` method will observe or drop that slot again, so
        // ownership of the value has moved cleanly to the caller with no
        // double-read and no double-drop.
        unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.len))) }
    }

    /// Returns a reference to the element at `index`, or `None` if
    /// `index` is out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        // SAFETY: `index < self.len`, and indices `[0, self.len)` are
        // always live, initialized `T` values within the current
        // allocation, so `self.ptr.as_ptr().add(index)` is a valid,
        // properly aligned pointer to a live `T`. The returned reference
        // borrows `self` immutably for its whole lifetime, and every
        // `&mut self` method requires exclusive access to `self`, so no
        // mutation can alias this reference while it is alive.
        unsafe { Some(&*self.ptr.as_ptr().add(index)) }
    }

    /// Number of elements currently stored.
    pub fn len(&self) -> usize {
        self.len
    }

    /// True if there are no elements stored.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Number of elements the current allocation can hold without
    /// reallocating.
    pub fn capacity(&self) -> usize {
        self.cap
    }
}

impl<T> Default for MiniVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for MiniVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        // SAFETY: `self.ptr` points to `self.len` consecutive, initialized
        // `T` values (indices `[0, len)` are always initialized -- `push`
        // only advances `len` after writing, `pop` only reads before
        // shrinking `len`). The allocation backing them never exceeds
        // `isize::MAX` bytes (checked in `with_capacity`/`grow`), and the
        // returned slice borrows `self` immutably for its lifetime, so
        // `slice::from_raw_parts`'s aliasing requirements are met.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> Drop for MiniVec<T> {
    fn drop(&mut self) {
        // Leak-on-panic tradeoff: `drop_in_place` on a slice has its own
        // built-in guard, so if one element's `Drop::drop` panics, every
        // *remaining* element is still dropped exactly once before the
        // panic propagates -- no double-drop, no UB. But that panic then
        // unwinds straight out of this function, skipping the `dealloc`
        // call below, so the backing allocation leaks (safe, but wasted
        // memory). `std::vec::Vec` avoids this by keeping its allocation
        // in a separate `RawVec` field: when unwinding past a panicking
        // element drop, Rust still runs every *other* field's destructor,
        // so `RawVec`'s `Drop` frees the buffer even though `Vec` itself
        // never returns normally. Closing this gap here would mean
        // splitting `MiniVec` the same way (see the Stretch goals in
        // README.md).
        if self.cap != 0 {
            // SAFETY: `self.ptr` points to `self.len` initialized, live
            // `T` values (see the `Deref` safety comment), and `drop`
            // takes `&mut self`, so there are no outstanding borrows of
            // any of them. `drop_in_place` on the slice
            // `[ptr, ptr + len)` therefore runs each element's destructor
            // exactly once.
            unsafe {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                    self.ptr.as_ptr(),
                    self.len,
                ));
            }

            let layout = Layout::array::<T>(self.cap).unwrap();
            // SAFETY: `self.ptr` was obtained from the global allocator
            // with a `Layout` for exactly `self.cap` elements of `T`
            // (established in `with_capacity` or the last `grow`, and
            // `self.cap` is never mutated anywhere else), so `layout`
            // matches the layout it was allocated with, as `dealloc`
            // requires. All elements were just dropped above, so no live
            // `T` is freed out from under a borrow.
            unsafe {
                alloc::dealloc(self.ptr.as_ptr().cast::<u8>(), layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_push_pop() {
        let mut v: MiniVec<i32> = MiniVec::new();
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());
        assert_eq!(v.pop(), None);

        v.push(10);
        v.push(20);
        v.push(30);

        assert_eq!(v.len(), 3);
        assert!(!v.is_empty());
        assert_eq!(v.get(0), Some(&10));
        assert_eq!(v.get(1), Some(&20));
        assert_eq!(v.get(2), Some(&30));
        assert_eq!(v.get(3), None);

        assert_eq!(v.pop(), Some(30));
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.len(), 1);
        assert_eq!(v.pop(), Some(10));
        assert_eq!(v.pop(), None);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_growth_amortization() {
        let mut v: MiniVec<u32> = MiniVec::new();
        assert_eq!(v.capacity(), 0);

        let mut growth_events = 0usize;
        let mut prev_cap = v.capacity();

        for i in 0..10_000u32 {
            v.push(i);
            assert!(
                v.capacity() >= v.len(),
                "capacity must never be smaller than len"
            );
            let cap = v.capacity();
            if cap != prev_cap {
                growth_events += 1;
                prev_cap = cap;
            }
        }

        assert_eq!(v.len(), 10_000);
        // Doubling growth means the number of reallocations is O(log n),
        // not O(n): for 10_000 pushes that's ~14 growth events. Bound it
        // comfortably below linear so a regression to "grow by 1" (which
        // would make it 10_000) fails loudly.
        assert!(
            growth_events <= 20,
            "expected O(log n) growth events for doubling growth, got {growth_events}"
        );
    }

    #[test]
    fn test_drop_called_exactly_once() {
        struct DropCounter<'a> {
            counter: &'a AtomicUsize,
        }

        impl<'a> Drop for DropCounter<'a> {
            fn drop(&mut self) {
                self.counter.fetch_add(1, Ordering::SeqCst);
            }
        }

        let counter = AtomicUsize::new(0);

        {
            let mut v: MiniVec<DropCounter> = MiniVec::new();
            for _ in 0..50 {
                v.push(DropCounter { counter: &counter });
            }
            assert_eq!(counter.load(Ordering::SeqCst), 0);

            // `pop` moves ownership out; the value must not be dropped
            // until the returned `DropCounter` itself is dropped.
            let popped = v.pop().expect("50 elements were pushed");
            assert_eq!(counter.load(Ordering::SeqCst), 0);
            drop(popped);
            assert_eq!(counter.load(Ordering::SeqCst), 1);

            assert_eq!(v.len(), 49);
        } // `v` dropped here: must drop the remaining 49 elements, each exactly once.

        assert_eq!(
            counter.load(Ordering::SeqCst),
            50,
            "every pushed element must be dropped exactly once (no double-drop, no leak)"
        );
    }

    #[test]
    fn test_deref_slice() {
        let mut v: MiniVec<i32> = MiniVec::new();
        for i in 1..=5 {
            v.push(i);
        }

        let slice: &[i32] = &v;
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
        assert_eq!(v.iter().sum::<i32>(), 15);
        assert_eq!(v.first(), Some(&1));
        assert_eq!(v.last(), Some(&5));
        assert_eq!(v.len(), 5);
    }

    #[test]
    #[should_panic(expected = "does not support zero-sized types")]
    fn test_zst_rejected() {
        let _v: MiniVec<()> = MiniVec::new();
    }
}
