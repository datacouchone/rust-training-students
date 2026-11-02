use case08b_memory::*;
#[test]
fn repeated_paths_share_bytes_without_merging_rows() {
    let mut b = Batch::new(Limits::default());
    let a = b.push(1, "/shared", 3).unwrap();
    let c = b.push(2, "/shared", 8).unwrap();
    assert_eq!(b.footprint().payload_bytes, 7);
    assert_eq!(b.footprint().unique_paths, 1);
    assert_eq!(b.get(a).unwrap().tenant, 1);
    assert_eq!(b.get(c).unwrap().tenant, 2);
    assert_eq!(b.total_bytes(), 11);
}
#[test]
fn hash_collision_does_not_alias_paths() {
    let mut b = Batch::with_hasher(Limits::default(), |_| 0);
    let a = b.push(1, "/first", 1).unwrap();
    let z = b.push(1, "/second", 2).unwrap();
    assert_eq!(b.get(a).unwrap().path, "/first");
    assert_eq!(b.get(z).unwrap().path, "/second");
    assert_eq!(b.footprint().unique_paths, 2);
}
#[test]
fn quota_rejection_is_transactional() {
    let mut b = Batch::new(Limits {
        bytes: 3,
        paths: 1,
        ..Limits::default()
    });
    let t = b.push(1, "/a", 5).unwrap();
    let old = b.footprint();
    assert_eq!(b.push(1, "/long", 7), Err(Error::PathLimit));
    assert_eq!(b.footprint(), old);
    assert_eq!(b.total_bytes(), 5);
    assert_eq!(b.get(t).unwrap().bytes, 5);
    assert!(b.push(2, "/a", 1).is_ok());
    let mut c = Batch::new(Limits {
        bytes: 1,
        ..Limits::default()
    });
    assert_eq!(c.push(0, "/a", 0), Err(Error::ByteLimit));
    assert_eq!(c.footprint().rows, 0);
    assert_eq!(c.footprint().unique_paths, 0);
}
#[test]
fn overflow_cannot_intern_a_rejected_path() {
    let mut b = Batch::new(Limits::default());
    b.push(1, "/x", u64::MAX).unwrap();
    let old = b.footprint();
    assert_eq!(b.push(1, "/new", 1), Err(Error::Overflow));
    assert_eq!(b.footprint(), old);
    assert_eq!(b.total_bytes(), u64::MAX);
}
#[test]
fn reset_invalidates_handles_and_reuses_small_capacity() {
    let mut b = Batch::new(Limits::default());
    let old = b.push(1, "/a", 1).unwrap();
    let cap = b.footprint().payload_capacity;
    b.reset().unwrap();
    assert_eq!(b.footprint().payload_capacity, cap);
    let now = b.push(2, "/b", 3).unwrap();
    assert_eq!(b.get(old), None);
    assert_eq!(b.get(now).unwrap().path, "/b");
    assert_eq!(b.total_bytes(), 3);
}
#[test]
fn large_batch_does_not_pin_oversized_storage() {
    let mut b = Batch::new(Limits {
        retain_bytes: 8,
        retain_rows: 1,
        retain_buckets: 0,
        ..Limits::default()
    });
    for i in 0..30 {
        b.push(1, &format!("/path/{i}"), 1).unwrap();
    }
    b.reset().unwrap();
    let f = b.footprint();
    assert_eq!(
        (f.payload_capacity, f.row_capacity, f.bucket_capacity),
        (0, 0, 0)
    );
    assert_eq!(f.payload_bytes, 0);
}
#[test]
fn unicode_and_zero_limits() {
    let mut b = Batch::new(Limits::default());
    let t = b.push(4, "/東京", 7).unwrap();
    assert_eq!(b.get(t).unwrap().path, "/東京");
    assert_eq!(b.footprint().payload_bytes, "/東京".len());
    let mut z = Batch::new(Limits {
        rows: 0,
        ..Limits::default()
    });
    assert_eq!(z.push(0, "/", 0), Err(Error::RowLimit));
    assert_eq!(b.push(0, "relative", 1), Err(Error::InvalidPath));
}
#[test]
fn multi_batch_differential_ownership() {
    let mut b = Batch::new(Limits::default());
    for n in [1, 100, 500] {
        let rows = fixture(n, 37);
        let tickets: Vec<_> = rows
            .iter()
            .map(|(t, p, v)| b.push(*t, p, *v).unwrap())
            .collect();
        for (ticket, (tenant, path, bytes)) in tickets.into_iter().zip(&rows) {
            assert_eq!(
                b.get(ticket),
                Some(View {
                    tenant: *tenant,
                    path,
                    bytes: *bytes
                })
            );
        }
        assert_eq!(b.total_bytes(), owned_baseline(&rows).1);
        b.reset().unwrap();
    }
}
