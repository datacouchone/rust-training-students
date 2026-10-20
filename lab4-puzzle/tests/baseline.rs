use lab4_puzzle::{Journal, Open, Prepared};

#[test]
fn empty_journal_has_fixed_capacity() {
    let journal = Journal::<String, 3>::new();
    assert_eq!(journal.capacity(), 3);
    assert_eq!(journal.len(), 0);
    assert_eq!(journal.iter().count(), 0);
}
#[test]
fn states_are_markers_and_zero_capacity_can_exist() {
    assert_eq!(std::mem::size_of::<Open>(), 0);
    assert_eq!(std::mem::size_of::<Prepared>(), 0);
    assert!(Journal::<String, 0>::new().is_empty());
}
