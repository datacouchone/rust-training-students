use case08a_profile::*;
fn rule(tenant: Option<u32>, prefix: &str, pool: u16) -> Rule {
    Rule {
        tenant,
        prefix: prefix.into(),
        pool,
    }
}
#[test]
fn segment_boundary_and_longest_prefix() {
    let p = [
        rule(None, "/", 0),
        rule(None, "/api", 1),
        rule(None, "/api/orders", 2),
    ];
    let i = Index::compile(&p).unwrap();
    for (path, want) in [
        ("/apiary", 0),
        ("/api", 1),
        ("/api/", 1),
        ("/api/orders/42", 2),
        ("/api/orders2", 1),
    ] {
        assert_eq!(i.route(7, path), Some(want), "{path}");
    }
}
#[test]
fn tenant_ties_do_not_override_longer_prefix() {
    let p = [
        rule(Some(7), "/", 9),
        rule(Some(7), "/api", 2),
        rule(None, "/api", 1),
        rule(None, "/api/deep", 3),
    ];
    let i = Index::compile(&p).unwrap();
    assert_eq!(i.route(7, "/api"), Some(2));
    assert_eq!(i.route(8, "/api"), Some(1));
    assert_eq!(i.route(7, "/api/deep/x"), Some(3));
    assert_eq!(i.route(8, "/unknown"), None);
}
#[test]
fn duplicates_and_ambiguous_prefixes_rejected() {
    for prefix in ["relative", "/api/", "/a//b", "/a?x", "/a#x"] {
        assert!(matches!(
            Index::compile(&[rule(None, prefix, 1)]),
            Err(Error::InvalidPrefix)
        ));
    }
    assert!(matches!(
        Index::compile(&[rule(None, "/a", 1), rule(None, "/a", 2)]),
        Err(Error::Duplicate)
    ));
    assert!(Index::compile(&[rule(None, "/a", 1), rule(Some(2), "/a", 2)]).is_ok());
}
#[test]
fn snapshot_owns_configuration() {
    let mut p = vec![rule(None, "/api", 1)];
    let i = Index::compile(&p).unwrap();
    p[0].pool = 8;
    p[0].prefix.clear();
    drop(p);
    assert_eq!(i.route(2, "/api/x"), Some(1));
}
#[test]
fn unicode_root_empty_and_invalid_requests() {
    let i = Index::compile(&[rule(None, "/é", 8)]).unwrap();
    assert_eq!(i.route(1, "/é/x"), Some(8));
    for p in ["", "api", "/é?x", "/é//x"] {
        assert_eq!(i.route(1, p), None);
    }
    assert_eq!(Index::compile(&[]).unwrap().route(1, "/"), None);
}
#[test]
fn differential_policies_and_workloads() {
    for count in [1, 7, 100, 513] {
        let (p, s) = fixture(503, count);
        let i = Index::compile(&p).unwrap();
        let requests = parse_requests(&s);
        assert_eq!(
            indexed_requests(&i, &requests),
            scan_requests(&p, &requests)
        );
        let mut reversed = p;
        reversed.reverse();
        let j = Index::compile(&reversed).unwrap();
        assert_eq!(
            indexed_requests(&i, &requests),
            indexed_requests(&j, &requests)
        );
    }
}
