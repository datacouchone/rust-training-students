//! Advanced MiniVec puzzle: insert and remove while preserving ownership.
//! All existing methods are provided; implement the two PUZZLE bodies.
//! This teaching collection rejects zero-sized types and may leak its backing
//! allocation if an element destructor panics during cleanup.

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

    /// Insert value before index; index == len appends.
    /// Panic if index > len, before changing the collection.
    pub fn insert(&mut self, index: usize, value: T) {
        // PUZZLE: validate, grow if needed, shift overlapping elements,
        // initialize the insertion slot, and publish the new length.
        // SAFETY: explain pointer bounds, overlap and ownership transfer.
        todo!("implement insert")
    }

    /// Remove index, returning its owned value and preserving element order.
    /// Panic if index >= len, before changing the collection.
    pub fn remove(&mut self, index: usize) -> T {
        // PUZZLE: validate, take ownership of the selected value, shift the
        // suffix left, and exclude the now-spare final slot from the live range.
        // SAFETY: explain why no element is read or destroyed twice.
        todo!("implement remove")
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
