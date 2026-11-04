use case08a_profile::*;
#[test]
fn supplied_baseline_and_fixture() {
    let (r, s) = fixture(20, 8);
    let req = parse_requests(&s);
    let out = scan_requests(&r, &req);
    assert_eq!(req.len(), 20);
    assert_eq!(summarize(&out).values().sum::<usize>(), 20);
    assert_eq!(linear(&r, 1, "/unknown"), Some(0));
}
