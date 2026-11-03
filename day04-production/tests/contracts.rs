use arrow::{
    array::{Array, UInt32Array},
    ipc::{reader::StreamReader, writer::StreamWriter},
};
use day04_production::{
    analytics::{self, Summary},
    encoding::{self, Encoder, Policy},
    routing::{self, Published, Snapshot},
};
use std::sync::{Arc, Barrier};

#[test]
fn reload_is_transactional_and_inflight_requests_pin_old_generation() {
    let service = Published::new(Snapshot::compile(1, routing::fixture(1, 32)).unwrap());
    let old = service.0.load_full();
    let mut invalid = routing::fixture(2, 32);
    invalid.push(invalid[0].clone());
    assert!(service.reload(2, invalid).is_err());
    assert_eq!(service.0.load().generation, 1);
    service.reload(2, routing::fixture(2, 32)).unwrap();
    assert_eq!(old.generation, 1);
    assert_eq!(old.route("tenant-7.example.net").unwrap().timeout_ms, 101);
    assert_eq!(service.0.load().generation, 2);
    assert!(service.0.load().route("unknown.example.net").is_none());
}
#[test]
fn readers_never_mix_route_and_policy_generations_during_reload() {
    let service = Arc::new(Published::new(
        Snapshot::compile(0, routing::fixture(0, 64)).unwrap(),
    ));
    let barrier = Arc::new(Barrier::new(5));
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let (service, barrier) = (service.clone(), barrier.clone());
            scope.spawn(move || {
                barrier.wait();
                for _ in 0..10_000 {
                    let pinned = service.0.load();
                    let route = pinned.route("tenant-17.example.net").unwrap();
                    assert_eq!(route.timeout_ms, 100 + pinned.generation as u32 % 1000);
                    assert_eq!(route.pool, 17);
                }
            });
        }
        barrier.wait();
        for generation in 1..200 {
            service
                .reload(generation, routing::fixture(generation, 64))
                .unwrap();
        }
    });
}
#[test]
fn ndjson_policies_preserve_bytes_and_escape_real_inputs() {
    let mut events = encoding::fixture(50, 7);
    events[0].path = "/orders/\"雪\n\\".into();
    let expected: Vec<u8> = events
        .iter()
        .flat_map(|e| {
            let mut b = serde_json::to_vec(e).unwrap();
            b.push(b'\n');
            b
        })
        .collect();
    let mut encoder = Encoder::new(1_000_000, 8192).unwrap();
    for policy in [Policy::Fresh, Policy::Reuse, Policy::Bounded] {
        for _ in 0..3 {
            assert_eq!(encoder.encode(&events, policy).unwrap(), expected);
        }
    }
    assert_eq!(
        expected
            .split(|b| *b == b'\n')
            .filter(|r| !r.is_empty())
            .count(),
        50
    );
}
#[test]
fn quota_error_exposes_no_partial_batch_and_recovers() {
    let small = encoding::fixture(1, 1);
    let needed = serde_json::to_vec(&small[0]).unwrap().len() + 1;
    let mut exact = Encoder::new(needed, needed).unwrap();
    assert_eq!(exact.encode(&small, Policy::Bounded).unwrap().len(), needed);
    assert!(exact
        .encode(&encoding::fixture(2, 1), Policy::Bounded)
        .is_err());
    assert_eq!(exact.encode(&small, Policy::Bounded).unwrap().len(), needed);
    let mut too_small = Encoder::new(needed - 1, 0).unwrap();
    assert!(too_small.encode(&small, Policy::Bounded).is_err());
    assert_eq!(too_small.retained_capacity(), 0);
}
#[test]
fn steady_burst_idle_policy_releases_large_and_reuses_small_buffers() {
    let (small, burst) = (encoding::fixture(8, 4), encoding::fixture(4096, 128));
    let mut encoder = Encoder::new(1_000_000, 8192).unwrap();
    encoder.encode(&small, Policy::Bounded).unwrap();
    let warm = encoder.retained_capacity();
    for _ in 0..30 {
        encoder.encode(&small, Policy::Bounded).unwrap();
        assert_eq!(encoder.retained_capacity(), warm);
    }
    encoder.encode(&burst, Policy::Bounded).unwrap();
    assert!(encoder.retained_capacity() > 8192);
    encoder.reset(Policy::Bounded);
    assert_eq!(encoder.retained_capacity(), 0);
    encoder.encode(&small, Policy::Bounded).unwrap();
    assert!(encoder.retained_capacity() <= 8192);
}
#[test]
fn arrow_nulls_status_bounds_and_widening_have_explicit_expected_result() {
    let batch = analytics::batch(
        vec![
            Some(u32::MAX),
            Some(u32::MAX),
            None,
            Some(90),
            Some(90),
            Some(90),
            Some(90),
        ],
        vec![
            Some(500),
            Some(599),
            Some(503),
            None,
            Some(499),
            Some(600),
            Some(503),
        ],
    );
    let expected = Summary {
        matched: 3,
        latency_us: 2 * u64::from(u32::MAX) + 90,
    };
    assert_eq!(analytics::kernels(&batch, 90).unwrap(), expected);
    assert_eq!(analytics::fused(&batch, 90).unwrap(), expected);
}
#[test]
fn arrow_offsets_tails_null_distributions_and_ipc_roundtrip() {
    for n in [0, 1, 7, 8, 9, 63, 64, 65, 4099] {
        for null_every in [1, 2, 17] {
            let full = analytics::fixture(n + 9, null_every, 3);
            for offset in 0..9 {
                let sliced = full.slice(offset, n);
                for threshold in [0, 50_000, u32::MAX] {
                    assert_eq!(
                        analytics::kernels(&sliced, threshold).unwrap(),
                        analytics::fused(&sliced, threshold).unwrap()
                    );
                }
                let mut bytes = Vec::new();
                {
                    let mut writer = StreamWriter::try_new(&mut bytes, &sliced.schema()).unwrap();
                    writer.write(&sliced).unwrap();
                    writer.finish().unwrap();
                }
                let restored = StreamReader::try_new(std::io::Cursor::new(bytes), None)
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    analytics::kernels(&restored, 50_000).unwrap(),
                    analytics::fused(&sliced, 50_000).unwrap()
                );
            }
        }
    }
}
#[test]
fn arrow_rejects_wrong_schema_in_both_plans() {
    let batch = arrow::record_batch::RecordBatch::try_from_iter([(
        "latency_us",
        Arc::new(UInt32Array::from(vec![1])) as Arc<dyn Array>,
    )])
    .unwrap();
    assert!(analytics::kernels(&batch, 0).is_err());
    assert!(analytics::fused(&batch, 0).is_err());
}
