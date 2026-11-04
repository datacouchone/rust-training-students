use case09_incident::*;
#[test]
fn producer_and_empty_accounting() {
    assert_eq!(frame(b"1:2"), vec![0, 0, 0, 3, 49, 58, 50]);
    assert_eq!(Decoder::default().buffered(), 0);
    assert!(Gateway::new(4).accounts().is_empty());
}
