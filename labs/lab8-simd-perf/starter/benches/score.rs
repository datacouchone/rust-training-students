//! Criterion bench comparing `score_scalar` vs `score_simd`.
//!
//! Run manually (not part of `cargo test`):
//!
//! ```text
//! cargo bench -p lab8-simd-perf-starter
//! ```
//!
//! This compiles from the very start (the `todo!()` bodies just panic if
//! actually *run*), so you can profile the given scalar baseline before
//! you write any SIMD code — see the README's "Steps" and "Profiling
//! workflow". `cargo flamegraph` runs this same bench binary.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lab8_simd_perf_starter::{score_scalar, score_simd};

/// Deterministic, non-random input generator (no need for true randomness
/// here — just enough variation that the compiler can't const-fold the
/// whole computation away).
fn make_input(len: usize) -> Vec<f32> {
    (0..len).map(|i| ((i % 173) as f32) - 20.0).collect()
}

fn bench_score(c: &mut Criterion) {
    let mut group = c.benchmark_group("score");

    // A small size (fits comfortably in L1), a medium size (typical batch),
    // and a large size (dominated by memory bandwidth, awkward tail).
    for &len in &[1_024usize, 65_536, 1_000_003] {
        let data = make_input(len);

        group.bench_with_input(BenchmarkId::new("scalar", len), &data, |b, d| {
            b.iter(|| score_scalar(black_box(d)));
        });

        // Panics with `todo!()` until you implement `score_simd` — comment
        // this block out if you want to profile the scalar baseline alone
        // before that.
        group.bench_with_input(BenchmarkId::new("simd", len), &data, |b, d| {
            b.iter(|| score_simd(black_box(d)));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_score);
criterion_main!(benches);
