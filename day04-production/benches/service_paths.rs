use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use day04_production::{
    analytics,
    encoding::{self, Encoder, Policy},
    routing::{self, Locked, Published, Snapshot},
};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
fn service_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_read");
    let locked = Locked(RwLock::new(Arc::new(
        Snapshot::compile(1, routing::fixture(1, 4096)).unwrap(),
    )));
    let published = Published::new(Snapshot::compile(1, routing::fixture(1, 4096)).unwrap());
    group.bench_function("rwlock_arc", |b| {
        b.iter(|| {
            black_box(
                locked
                    .pin()
                    .route(black_box("tenant-17.example.net"))
                    .cloned(),
            )
        })
    });
    group.bench_function("arc_swap_owned", |b| {
        b.iter(|| {
            black_box(
                published
                    .0
                    .load_full()
                    .route(black_box("tenant-17.example.net"))
                    .cloned(),
            )
        })
    });
    group.bench_function("arc_swap_guard", |b| {
        b.iter(|| {
            black_box(
                published
                    .0
                    .load()
                    .route(black_box("tenant-17.example.net"))
                    .cloned(),
            )
        })
    });
    group.finish();
    let mut group = c.benchmark_group("ndjson_steady_state");
    for rows in [32, 4096] {
        let events = encoding::fixture(rows, 128);
        group.throughput(Throughput::Elements(rows as u64));
        for policy in [Policy::Fresh, Policy::Reuse, Policy::Bounded] {
            let mut encoder = Encoder::new(4 * 1024 * 1024, 512 * 1024).unwrap();
            group.bench_with_input(
                BenchmarkId::new(format!("{policy:?}"), rows),
                &events,
                |b, input| {
                    b.iter(|| {
                        black_box(encoder.encode(black_box(input), policy).unwrap());
                    })
                },
            );
        }
    }
    group.finish();
    let mut group = c.benchmark_group("arrow_aggregate");
    for (rows, null_every, error_every) in [
        (128, 17, 19),
        (65_539, 17, 19),
        (65_539, 2, 2),
        (65_539, 97, 1),
    ] {
        let batch = analytics::fixture(rows, null_every, error_every);
        assert_eq!(
            analytics::kernels(&batch, 50_000).unwrap(),
            analytics::fused(&batch, 50_000).unwrap()
        );
        group.throughput(Throughput::Elements(rows as u64));
        let workload = format!("n{rows}_null{null_every}_error{error_every}");
        group.bench_with_input(
            BenchmarkId::new("kernels", &workload),
            &batch,
            |b, input| {
                b.iter(|| {
                    black_box(analytics::kernels(black_box(input), black_box(50_000)).unwrap())
                })
            },
        );
        group.bench_with_input(BenchmarkId::new("fused", &workload), &batch, |b, input| {
            b.iter(|| black_box(analytics::fused(black_box(input), black_box(50_000)).unwrap()))
        });
    }
    group.finish();
}
criterion_group! { name = benches; config = Criterion::default().sample_size(30).warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(2)); targets = service_paths }
criterion_main!(benches);
