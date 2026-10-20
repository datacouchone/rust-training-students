use miri_puzzle::{Audit, Batch, OneShot};

#[test]
fn empty_constructors_need_no_initialized_payload() {
    let batch = Batch::<String, 4>::new();
    assert_eq!(
        (batch.len(), batch.capacity(), batch.is_empty()),
        (0, 4, true)
    );
    let zero = Batch::<String, 0>::new();
    assert_eq!(zero.capacity(), 0);
    let _handoff = OneShot::<String>::new();
}

#[test]
fn initial_accounting_is_empty_and_consistent() {
    let snapshot = Audit::default().snapshot();
    assert_eq!(snapshot.completed, 0);
    assert_eq!(snapshot.total_bytes, 0);
    assert!(snapshot.entries.is_empty());
}
