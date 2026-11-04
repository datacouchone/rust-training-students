use case06_admission::*;
#[test]
fn budgets_start_empty_and_independent() {
    let a = Budget::new(32);
    let b = Budget::new(8);
    assert_eq!((a.used(), a.limit()), (0, 32));
    assert_eq!((b.used(), b.limit()), (0, 8));
}
