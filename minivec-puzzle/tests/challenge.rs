use minivec_puzzle::MiniVec;
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

#[test]
fn insert_positions_and_growth() {
    let mut v = MiniVec::with_capacity(4);
    v.push(10);
    v.push(30);
    let capacity = v.capacity();
    v.insert(1, 20);
    assert_eq!(&*v, &[10, 20, 30]);
    assert_eq!(v.capacity(), capacity);
    v.insert(0, 5);
    let end = v.len();
    v.insert(end, 40);
    assert_eq!(&*v, &[5, 10, 20, 30, 40]);
    assert_eq!(v.len(), 5);
    assert!(v.capacity() > capacity);
}

#[test]
fn empty_and_single_element() {
    let mut v = MiniVec::new();
    v.insert(0, 99);
    let capacity = v.capacity();
    assert_eq!(v.remove(0), 99);
    assert!(v.is_empty());
    assert_eq!(v.capacity(), capacity);
    v.insert(0, 7);
    assert_eq!(v.pop(), Some(7));
}

#[test]
fn remove_preserves_order_and_capacity() {
    let mut v = MiniVec::with_capacity(8);
    for n in [10, 20, 30, 40, 50] { v.push(n); }
    let capacity = v.capacity();
    assert_eq!(v.remove(2), 30);
    assert_eq!(&*v, &[10, 20, 40, 50]);
    assert_eq!(v.remove(0), 10);
    assert_eq!(v.remove(v.len() - 1), 50);
    assert_eq!(&*v, &[20, 40]);
    assert_eq!(v.capacity(), capacity);
}

#[test]
fn strings_are_moved_without_cloning() {
    let mut v = MiniVec::new();
    v.insert(0, String::from("alpha"));
    v.push(String::from("gamma"));
    v.insert(1, String::from("beta"));
    let removed = v.remove(1);
    assert_eq!(removed, "beta");
    assert_eq!(v.get(0).map(String::as_str), Some("alpha"));
    assert_eq!(v.get(1).map(String::as_str), Some("gamma"));
    drop(v);
    assert_eq!(removed, "beta", "removed value belongs to its caller");
}

struct Tracked { id: usize, drops: Rc<Vec<Cell<usize>>> }
impl Drop for Tracked {
    fn drop(&mut self) { self.drops[self.id].set(self.drops[self.id].get() + 1); }
}

#[test]
fn every_owned_value_is_dropped_exactly_once() {
    let drops = Rc::new((0..4).map(|_| Cell::new(0)).collect::<Vec<_>>());
    let item = |id| Tracked { id, drops: Rc::clone(&drops) };
    let mut v = MiniVec::with_capacity(4);
    v.push(item(0));
    v.push(item(2));
    v.insert(1, item(1));
    v.insert(0, item(3));
    assert!(drops.iter().all(|n| n.get() == 0), "shifting must not drop values");
    let removed = v.remove(2);
    assert_eq!(removed.id, 1);
    assert!(drops.iter().all(|n| n.get() == 0));
    drop(v);
    assert_eq!(drops[0].get(), 1);
    assert_eq!(drops[1].get(), 0, "caller still owns removed value");
    assert_eq!(drops[2].get(), 1);
    assert_eq!(drops[3].get(), 1);
    drop(removed);
    assert!(drops.iter().all(|n| n.get() == 1));
}

#[test]
fn invalid_indices_leave_collection_unchanged() {
    let mut v = MiniVec::with_capacity(1);
    v.push(10);
    let capacity = v.capacity();
    assert!(catch_unwind(AssertUnwindSafe(|| v.insert(2, 99))).is_err());
    assert_eq!(&*v, &[10]);
    assert_eq!(v.capacity(), capacity, "validate before growing");
    assert!(catch_unwind(AssertUnwindSafe(|| v.remove(1))).is_err());
    assert_eq!(&*v, &[10]);
    assert_eq!(v.capacity(), capacity);
}

#[test]
fn remove_from_empty_panics_without_allocating() {
    let mut v = MiniVec::<u32>::new();
    assert!(catch_unwind(AssertUnwindSafe(|| v.remove(0))).is_err());
    assert_eq!(v.len(), 0);
    assert_eq!(v.capacity(), 0);
}

#[test]
fn repeated_overlapping_moves_match_vec() {
    let mut actual = MiniVec::new();
    let mut expected = Vec::new();
    for n in 0..96 {
        let index = (n * 7) % (expected.len() + 1);
        actual.insert(index, n);
        expected.insert(index, n);
        assert_eq!(&*actual, expected.as_slice());
    }
    for n in 0..96 {
        let index = (n * 11) % expected.len();
        assert_eq!(actual.remove(index), expected.remove(index));
        assert_eq!(&*actual, expected.as_slice());
    }
}
