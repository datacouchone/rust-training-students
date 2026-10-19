//! Complete reference solution for the MiniVec insertion/removal puzzle.
//!
//! Read insert() and remove() alongside the starter in src/lib.rs. This file
//! is a reference, not an automatically compiled Cargo target. To check your
//! implementation, copy only the relevant method bodies into the starter,
//! save, and run the commands in README.md. Keep the existing tests unchanged.
//!
//! Vocabulary used below:
//! - T is the element type: an integer, String, or another owned value.
//! - len counts initialized values owned by this collection.
//! - cap counts slots in the allocation; spare slots are not live T values.
//! - ptr identifies that allocation, or an aligned placeholder when cap is zero.
//! - Moving bytes is not cloning ownership. The live range determines which
//!   locations still own values and which locations must never be dropped.
//! - Unsafe blocks require a validity argument; they do not disable Rust's
//!   remaining type and borrow checks.
//!
//! Scope: zero-sized types are rejected. Normal cleanup is implemented; an
//! element destructor that panics can cause this teaching implementation to
//! leak its backing allocation. Passing tests/Miri is evidence for tested
//! executions, not a proof covering every possible safe caller.

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
        assert_ne!(
            mem::size_of::<T>(),
            0,
            "MiniVec<T> does not support zero-sized types"
        );

        if capacity == 0 {
            return Self::new();
        }

        let layout = Layout::array::<T>(capacity).expect("capacity overflow");

        // SAFETY: capacity is positive and T is non-zero-sized.
        // Layout::array supplied a valid, non-zero allocation layout.
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

        let raw_ptr = if self.cap == 0 {
            // SAFETY: new_layout describes one non-zero-sized T.
            // There is no existing allocation to resize.
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            // SAFETY: ptr owns an allocation from this allocator with old_layout.
            // The new size is non-zero and the alignment is unchanged.
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

        // SAFETY: there is now a spare allocated slot at index len.
        // It is aligned and not currently a live T.
        // ptr::write initializes it without dropping uninitialized contents.
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), value);
        }

        self.len += 1;
    }

    /// Insert value before index, preserving order. index == len appends.
    pub fn insert(&mut self, index: usize, value: T) {
        // A collection of length 3 accepts insertion at 0, 1, 2 or 3.
        // Check before growth: an invalid request must not change ptr/cap/len.
        // If this panics, Rust drops the still-owned value argument normally.
        assert!(index <= self.len, "index out of bounds");

        // Inserting one value needs a spare slot. Only grow if every slot is
        // occupied. grow may relocate the allocation, so do not obtain base
        // or any element pointer until AFTER this call has returned.
        if self.len == self.cap {
            self.grow();
        }

        // SAFETY: growth establishes len < cap. index <= len, so the source
        // suffix [index, len) contains initialized values, and the destination
        // [index + 1, len + 1) fits in this same allocation. Both pointers are
        // aligned for T. These ranges can overlap; ptr::copy permits overlap.
        // No destructor, allocation or user callback runs between relocation
        // and publishing the new ownership range below.
        unsafe {
            let base = self.ptr.as_ptr();

            // Pointer add counts T ELEMENTS, not bytes. For u32, moving one
            // element advances four bytes on our target.
            //
            // Example: [A, B, C, spare], index 1, len 3.
            // Source is B,C (len-index = 2 values), destination starts at 2.
            // ptr::copy has memmove-style overlap behavior. It does not call
            // Clone or Drop, so this also works for owning values like String.
            // Temporarily duplicated bits do not give us two logical owners.
            ptr::copy(base.add(index), base.add(index + 1), self.len - index);

            // The old index's value has been relocated to index+1. Initialize
            // the gap with value without dropping the stale bits at index.
            // Ordinary assignment here could drop those bits and invalidate
            // the relocated owner, causing a later use-after-free/double-free.
            ptr::write(base.add(index), value);
        }

        // Now, and only now, all slots [0, old_len+1) own initialized values.
        // Capacity describes storage; length records the newly live value.
        // For append, the shift count was zero and this still works.
        self.len += 1;
    }

    /// Remove index, transfer its value to the caller, preserve remaining order.
    pub fn remove(&mut self, index: usize) -> T {
        // Unlike insert, index == len is invalid: no value lives there.
        // Validate before subtracting lengths or touching a pointer.
        assert!(index < self.len, "index out of bounds");

        // SAFETY: index names one initialized value. The source suffix
        // [index+1, len) and its destination [index, len-1) lie within the
        // same allocation and may overlap. No user code runs during this
        // sequence. Shortening len excludes the stale final slot before the
        // method returns, so it will not be read or dropped as a second owner.
        unsafe {
            let base = self.ptr.as_ptr();

            // ptr::read transfers the selected T into a local owned value
            // without destroying it or clearing the old bytes. The returned
            // value must remain alive independently of this collection.
            let result = ptr::read(base.add(index));

            // Example: [A, B, C, D], removing index 1.
            // Move C,D left into positions 1,2: len-index-1 = 2 elements.
            // Do not drop the overwritten bytes: B is already owned by result,
            // and any old destination values are being relocated, not removed.
            // When removing the last element, the source is one-past the live
            // range and the copy count is zero; no source element is read.
            ptr::copy(base.add(index + 1), base.add(index), self.len - index - 1);

            // Old trailing bits may duplicate the last moved value. Excluding
            // that slot from [0,len) prevents double-drop when MiniVec dies.
            // remove never shrinks capacity or reallocates the backing buffer.
            self.len -= 1;

            // No semicolon: this expression returns the owned T to the caller.
            // Its eventual destructor is the caller's responsibility.
            result
        }
    }

    /// Removes and returns the last element, or `None` if the vector is
    /// empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        // SAFETY: this index was the final initialized element.
        // Shortening len removes it from MiniVec's live range before
        // ptr::read transfers ownership to the caller.
        unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.len))) }
    }

    /// Returns a reference to the element at `index`, or `None` if
    /// `index` is out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        // SAFETY: index is within the initialized, owned prefix.
        // The pointer is aligned and belongs to the current allocation.
        // The returned shared reference is tied to the borrow of self.
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
        // SAFETY: ptr is non-null and aligned, even when empty.
        // The first len elements are initialized within one allocation.
        // The validated allocation bounds its size, and the slice borrows self.
        unsafe {
            std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> Drop for MiniVec<T> {
    fn drop(&mut self) {
        // If an element destructor panics, this simplified implementation
        // can skip deallocation and leak the buffer.
        if self.cap != 0 {
            // SAFETY: the first len elements are initialized and owned.
            // Exclusive access permits destroying them in place.
            unsafe {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                    self.ptr.as_ptr(),
                    self.len,
                ));
            }

            let layout = Layout::array::<T>(self.cap).unwrap();

            // SAFETY: ptr owns this allocation with this exact layout.
            // The live elements have been destroyed.
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
