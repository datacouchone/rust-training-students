//! Transactional bounded journal: typestate, const generics, ownership and lending.
//! Reference solution: all six implementation tasks A-F are complete.
#![forbid(unsafe_code)]

use std::marker::PhantomData;

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Open {}
    impl Sealed for super::Prepared {}
}
pub trait TxState: sealed::Sealed {}
pub struct Open;
pub struct Prepared;
impl TxState for Open {}
impl TxState for Prepared {}

pub struct Journal<T, const N: usize> {
    slots: [Option<T>; N],
    // Unlike the core lab, head is the OLDEST retained element, not next write.
    head: usize,
    len: usize,
}

impl<T, const N: usize> Journal<T, N> {
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| None),
            head: 0,
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub const fn capacity(&self) -> usize {
        N
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).map(|i| self.slots[(self.head + i) % N].as_ref().unwrap())
    }
    pub fn begin(&mut self) -> Transaction<'_, T, N, Open> {
        Transaction {
            journal: self,
            pending: Vec::new(),
            state: PhantomData,
        }
    }
    pub fn windows_mut(&mut self, width: usize) -> WindowsMut<'_, T, N> {
        WindowsMut {
            journal: self,
            width,
            pos: 0,
        }
    }

    // D: called only by a nonempty prepared transaction, so N > 0.
    // Return an evicted owner only when full. Never drop it here.
    fn push_committed(&mut self, value: T) -> Option<T> {
        assert!(
            N > 0,
            "only a nonempty prepared transaction reaches this method"
        );
        if self.len < N {
            let index = (self.head + self.len) % N;
            self.slots[index] = Some(value);
            self.len += 1;
            None
        } else {
            // Taking the old owner out avoids dropping it during publication.
            let old = self.slots[self.head].replace(value);
            self.head = (self.head + 1) % N;
            old
        }
    }
}
impl<T, const N: usize> Default for Journal<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

// Dropping either state cancels staging naturally: pending owns the new values;
// the exclusive journal borrow is released; committed slots were never changed.
pub struct Transaction<'j, T, const N: usize, State: TxState> {
    journal: &'j mut Journal<T, N>,
    pending: Vec<T>,
    state: PhantomData<State>,
}
impl<T, const N: usize, State: TxState> Transaction<'_, T, N, State> {
    pub fn staged_len(&self) -> usize {
        self.pending.len()
    }
}
impl<'j, T, const N: usize> Transaction<'j, T, N, Open> {
    // A: allow at most N staged items. On refusal return the exact input owner.
    // Staging must not modify the journal. N = 0 must return Err(value).
    pub fn stage(&mut self, value: T) -> Result<(), T> {
        if self.pending.len() == N {
            return Err(value);
        }
        self.pending.push(value);
        Ok(())
    }
    // B: an empty transaction is not prepared: return the ORIGINAL transaction.
    // Success consumes Open and transfers both fields to Prepared.
    pub fn prepare(self) -> Result<Transaction<'j, T, N, Prepared>, Self> {
        if self.pending.is_empty() {
            return Err(self);
        }
        Ok(Transaction {
            journal: self.journal,
            pending: self.pending,
            state: PhantomData,
        })
    }
}
impl<T, const N: usize> Transaction<'_, T, N, Prepared> {
    // C: publish in staging order; return evicted owners oldest-first.
    // Reserve output space before the first journal change. No user callbacks.
    pub fn commit(self) -> Vec<T> {
        let Transaction {
            journal, pending, ..
        } = self;
        // Reserve before changing the journal, so growing the returned vector
        // cannot interrupt a partially applied commit.
        let mut evicted = Vec::with_capacity(pending.len());
        for value in pending {
            if let Some(old) = journal.push_committed(value) {
                evicted.push(old);
            }
        }
        evicted
    }
}

pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}
pub struct WindowsMut<'j, T, const N: usize> {
    journal: &'j mut Journal<T, N>,
    width: usize,
    pos: usize,
}
// A logical window may wrap across the physical end, so it is NOT a contiguous
// mutable slice. Each get_mut lends just one element through this window.
pub struct Window<'a, T, const N: usize> {
    journal: &'a mut Journal<T, N>,
    start: usize,
    width: usize,
}
impl<T, const N: usize> Window<'_, T, N> {
    pub fn len(&self) -> usize {
        self.width
    }
    pub fn is_empty(&self) -> bool {
        self.width == 0
    }
    // F: reject index >= width BEFORE index arithmetic, then map logical to
    // physical. The returned borrow must be tied to this get_mut call.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.width {
            return None;
        }
        let logical = self.start.checked_add(index)?;
        if logical >= self.journal.len {
            return None;
        }
        let physical = (self.journal.head + logical) % N;
        self.journal.slots[physical].as_mut()
    }
}
impl<T, const N: usize> LendingIterator for WindowsMut<'_, T, N> {
    type Item<'a>
        = Window<'a, T, N>
    where
        Self: 'a;
    // E: width 0 or a window that does not fully fit yields None.
    // Use checked arithmetic. Advance by one; never copy/linearize the journal.
    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.width == 0 {
            return None;
        }
        let end = self.pos.checked_add(self.width)?;
        if end > self.journal.len {
            return None;
        }
        let start = self.pos;
        self.pos += 1;
        Some(Window {
            journal: &mut *self.journal,
            start,
            width: self.width,
        })
    }
}
