#[test]
fn ordinary_owned_values_are_valid() {
    let packet = vec![10u32, 20, 30];
    assert_eq!(packet.get(2), Some(&30));
    assert_eq!(packet.get(3), None);
}

#[test]
fn scoped_thread_finishes_before_owner_disappears() {
    let text = String::from("storage");
    std::thread::scope(|scope| {
        scope.spawn(|| assert_eq!(text.len(), 7));
    });
    assert_eq!(text, "storage");
}
