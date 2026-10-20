use lab4_puzzle::{Journal, LendingIterator};
use std::{cell::Cell, rc::Rc};

fn append<T, const N: usize>(
    journal: &mut Journal<T, N>,
    values: impl IntoIterator<Item = T>,
) -> Vec<T> {
    let mut tx = journal.begin();
    for value in values {
        assert!(tx.stage(value).is_ok());
    }
    match tx.prepare() {
        Ok(ready) => ready.commit(),
        Err(_) => panic!("the test staged a nonempty batch"),
    }
}
fn values<const N: usize>(j: &Journal<i32, N>) -> Vec<i32> {
    j.iter().copied().collect()
}

#[test]
fn stage_returns_the_original_rejected_owner() {
    let mut j = Journal::<Box<u32>, 1>::new();
    let mut tx = j.begin();
    assert!(tx.stage(Box::new(1)).is_ok());
    let rejected = Box::new(99);
    let address = &*rejected as *const u32;
    let returned = tx.stage(rejected).unwrap_err();
    assert_eq!(&*returned as *const u32, address);
    assert_eq!(tx.staged_len(), 1);
}
#[test]
fn stage_zero_capacity_keeps_the_value() {
    let mut j = Journal::<String, 0>::new();
    let mut tx = j.begin();
    assert_eq!(tx.stage(String::from("owned")), Err(String::from("owned")));
    assert_eq!(tx.staged_len(), 0);
}
#[test]
fn stage_empty_prepare_returns_a_reusable_transaction() {
    let mut j = Journal::<i32, 2>::new();
    let mut tx = match j.begin().prepare() {
        Err(tx) => tx,
        Ok(_) => panic!("an empty transaction must not prepare"),
    };
    assert!(tx.stage(7).is_ok());
    assert_eq!(tx.staged_len(), 1);
    // Cancellation must not publish 7.
    drop(tx);
    assert!(j.is_empty());
}
#[test]
fn stage_cancel_open_preserves_the_committed_journal() {
    let mut j = Journal::<i32, 3>::new();
    // No call to commit: this stage test is independent of C/D.
    let mut tx = j.begin();
    assert!(tx.stage(1).is_ok());
    assert!(tx.stage(2).is_ok());
    drop(tx);
    assert!(j.is_empty());
}
#[test]
fn stage_cancel_prepared_does_not_publish() {
    let mut j = Journal::<String, 2>::new();
    let mut tx = j.begin();
    assert!(tx.stage(String::from("pending")).is_ok());
    let ready = tx.prepare().ok().expect("nonempty prepares");
    assert_eq!(ready.staged_len(), 1);
    drop(ready);
    assert!(j.is_empty());
}

#[test]
fn journal_commits_in_staging_order() {
    let mut j = Journal::<i32, 3>::new();
    assert!(append(&mut j, [1, 2]).is_empty());
    assert_eq!(values(&j), [1, 2]);
    assert_eq!(j.len(), 2);
}
#[test]
fn journal_wrap_returns_evictions_oldest_first() {
    let mut j = Journal::<i32, 3>::new();
    append(&mut j, [1, 2, 3]);
    assert_eq!(append(&mut j, [4, 5]), [1, 2]);
    assert_eq!(values(&j), [3, 4, 5]);
    assert_eq!(append(&mut j, [6, 7, 8]), [3, 4, 5]);
    assert_eq!(values(&j), [6, 7, 8]);
    assert_eq!(j.len(), 3);
}
#[test]
fn journal_capacity_one_and_noncopy_values() {
    let mut j = Journal::<String, 1>::new();
    assert!(append(&mut j, [String::from("first")]).is_empty());
    assert_eq!(append(&mut j, [String::from("next")]), ["first"]);
    assert_eq!(j.iter().map(String::as_str).collect::<Vec<_>>(), ["next"]);
}
#[test]
fn journal_panic_before_commit_rolls_back_staging() {
    let mut j = Journal::<i32, 3>::new();
    append(&mut j, [10, 20]);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut tx = j.begin();
        assert!(tx.stage(99).is_ok());
        let _ready = tx.prepare().ok().expect("nonempty");
        panic!("cancel the prepared transaction");
    }));
    assert!(result.is_err());
    assert_eq!(values(&j), [10, 20]);
}
#[test]
fn journal_ownership_is_released_exactly_once() {
    struct Item(Rc<Cell<usize>>);
    impl Drop for Item {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    let mut j = Journal::<Item, 2>::new();
    append(&mut j, [Item(drops.clone()), Item(drops.clone())]);
    let evicted = append(&mut j, [Item(drops.clone())]);
    assert_eq!(drops.get(), 0);
    drop(evicted);
    assert_eq!(drops.get(), 1);
    {
        let mut tx = j.begin();
        assert!(tx.stage(Item(drops.clone())).is_ok());
    }
    assert_eq!(drops.get(), 2);
    drop(j);
    assert_eq!(drops.get(), 4);
}

#[test]
fn windows_mutate_logical_order_across_a_physical_wrap() {
    let mut j = Journal::<i32, 3>::new();
    append(&mut j, [1, 2, 3]);
    append(&mut j, [4]); // physical [4,2,3], logical [2,3,4]
    let mut windows = j.windows_mut(2);
    {
        let mut first = windows.next().expect("first");
        assert_eq!(first.len(), 2);
        *first.get_mut(0).unwrap() += 10;
        *first.get_mut(1).unwrap() += 100;
        assert!(first.get_mut(2).is_none());
        assert!(first.get_mut(usize::MAX).is_none());
    }
    {
        let mut second = windows.next().expect("second");
        assert_eq!(*second.get_mut(0).unwrap(), 103);
        *second.get_mut(1).unwrap() += 1000;
    }
    assert!(windows.next().is_none());
    assert_eq!(values(&j), [12, 103, 1004]);
}
#[test]
fn windows_reject_zero_oversized_and_overflowing_widths() {
    let mut j = Journal::<i32, 2>::new();
    append(&mut j, [1, 2]);
    assert!(j.windows_mut(0).next().is_none());
    assert!(j.windows_mut(3).next().is_none());
    assert!(j.windows_mut(usize::MAX).next().is_none());
    let mut empty = Journal::<i32, 0>::new();
    assert!(empty.windows_mut(1).next().is_none());
}
#[test]
fn windows_width_one_visits_each_retained_value_once() {
    let mut j = Journal::<i32, 3>::new();
    append(&mut j, [1, 2, 3]);
    let mut windows = j.windows_mut(1);
    let mut count = 0;
    while let Some(mut window) = windows.next() {
        *window.get_mut(0).unwrap() *= 2;
        count += 1;
    }
    assert_eq!(count, 3);
    assert_eq!(values(&j), [2, 4, 6]);
}
#[test]
fn windows_edit_owned_values_without_cloning() {
    let mut j = Journal::<String, 2>::new();
    append(&mut j, [String::from("a"), String::from("b")]);
    let mut windows = j.windows_mut(2);
    {
        let mut window = windows.next().unwrap();
        window.get_mut(0).unwrap().push('!');
        window.get_mut(1).unwrap().push('?');
    }
    assert!(windows.next().is_none());
    assert_eq!(
        j.iter().map(String::as_str).collect::<Vec<_>>(),
        ["a!", "b?"]
    );
}
