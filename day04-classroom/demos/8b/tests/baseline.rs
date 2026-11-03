use case08b_memory::*;
#[test]
fn empty_batch_and_owned_baseline() {
    let b = Batch::new(Limits::default());
    assert_eq!(b.footprint().rows, 0);
    let rows = fixture(10, 2);
    let (v, total) = owned_baseline(&rows);
    assert_eq!(v, rows);
    assert_eq!(total, 45);
}
