//! `MiniVec<T>`: a minimal, safe abstraction over a raw heap allocation.
//!
//! `new` is implemented for you so you can see the ZST guard and the
//! "an empty vec never allocates" invariant established via
//! `NonNull::dangling()`. Every other method's body is `todo!()`.
//!
//! Fill them in using `std::alloc::{alloc, realloc, dealloc}` and
//! `Layout`. Wherever you see `// SAFETY: <fill in>`, that marks a spot
//! where you are about to write (or are already looking at) an `unsafe`
//! block: replace `<fill in>` with a real argument for why the operation
//! immediately below is sound, referencing the specific invariants of
//! `MiniVec` (what `len`, `cap`, and `ptr` mean and how they relate) that
//! make it safe. "Trust me" is not a safety comment.
//!
//! The test module at the bottom is the same test suite your implementation must
//! pass — run `cargo test` as you go; every test will panic on `todo!()`
//! until you implement the method it needs.

#![allow(unused_imports, dead_code)]

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
    /// Panics if `T` is a zero-sized type — supporting ZSTs correctly
    /// needs special-cased pointer arithmetic throughout, which is out of
    /// scope here, so we reject them explicitly instead of silently doing
    /// the wrong thing.
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
        // TODO:
        //   1. Guard against ZSTs the same way `new` does.
        //   2. If `capacity == 0`, just delegate to `Self::new()`.
        //   3. Otherwise, build a `Layout::array::<T>(capacity)` and
        //      allocate. `Layout::array` already returns `Err` if the
        //      requested size would overflow `isize::MAX`, so no separate
        //      assert is needed for that.
        //   4. Check the returned pointer against null before building
        //      the `NonNull`; call `alloc::handle_alloc_error` if it is.
        //
        // SAFETY: <fill in>
        todo!()
    }

    /// Grows the backing allocation: from `0` to `1` element, or by
    /// doubling otherwise. Called by `push` whenever `len == cap`.
    fn grow(&mut self) {
        // TODO: compute `new_cap` (1 if `cap == 0`, else `cap * 2`) and the
        // corresponding `Layout::array::<T>(new_cap)`. Call `alloc::alloc`
        // if there was no previous allocation, or `alloc::realloc`
        // (passing the *old* layout) otherwise. Update `self.ptr` and
        // `self.cap` from the result, handling allocation failure.
        //
        // SAFETY: <fill in>
        todo!()
    }

    /// Appends `value` to the end, growing the backing allocation first if
    /// there is no spare capacity.
    pub fn push(&mut self, value: T) {
        // TODO: call `self.grow()` when `self.len == self.cap`, then
        // `ptr::write` `value` into the slot at offset `self.len` and
        // increment `self.len`.
        //
        // SAFETY: <fill in>
        todo!()
    }

    /// Removes and returns the last element, or `None` if the vector is
    /// empty.
    pub fn pop(&mut self) -> Option<T> {
        // TODO: if `self.len == 0`, return `None`. Otherwise decrement
        // `self.len` first, then `ptr::read` the value out of the slot
        // that index now refers to.
        //
        // SAFETY: <fill in>
        todo!()
    }

    /// Returns a reference to the element at `index`, or `None` if
    /// `index` is out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        // TODO: bounds-check `index` against `self.len`, then dereference
        // the pointer at that offset.
        //
        // SAFETY: <fill in>
        todo!()
    }

    /// Number of elements currently stored.
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True if there are no elements stored.
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Number of elements the current allocation can hold without
    /// reallocating.
    pub fn capacity(&self) -> usize {
        todo!()
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
        // TODO: build a slice over the `self.len` initialized elements
        // starting at `self.ptr`.
        //
        // SAFETY: <fill in>
        todo!()
    }
}

impl<T> Drop for MiniVec<T> {
    fn drop(&mut self) {
        // TODO: if `self.cap != 0`:
        //   1. Drop every live element (indices `[0, self.len)`) — see
        //      `ptr::drop_in_place` on a slice built with
        //      `ptr::slice_from_raw_parts_mut`.
        //   2. Deallocate the backing buffer with the same `Layout` it was
        //      allocated with (`Layout::array::<T>(self.cap)`).
        //
        // SAFETY: <fill in>
        todo!()
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
