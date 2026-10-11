//! Lab 8 — SIMD + arena allocation + profiling.
//!
//! Scenario: a CPU-bound "log-metrics scorer". Each log record contributes
//! four `f32` metrics — `[latency, cpu, mem, err_rate]` — packed back to
//! back into one flat `&[f32]` (so a batch of `n` records is `4 * n` floats
//! long). The scorer normalizes each metric into roughly `[0, 1]`, clamps
//! it (bad/garbage readings can be negative or far above the expected
//! range), and folds it into a weighted sum where the weight cycles through
//! the four metric columns.
//!
//! [`score_scalar`] below is given, complete, and deliberately naive — it's
//! the baseline you'll profile and then beat. Your job is:
//!
//! 1. `score_simd` — the same math, vectorized with `std::simd::f32x8`
//!    (nightly-only: add `#![feature(portable_simd)]` at the top of this
//!    file once you start writing it) plus a scalar tail for whatever
//!    doesn't divide evenly into 8-lane chunks.
//! 2. `Arena` — a small bump allocator. See the lab README's "Steps" for a
//!    guided walkthrough of the design decisions (how `alloc(&self, ..)`
//!    can hand out `&T` without `&mut self`, and what that implies about
//!    `reset`).
//!
//! See the lab README for the full task, TDD workflow, and done-criteria.

/// Metrics are expected in roughly the `0..100` range; this scales them
/// down to roughly `0..1` before clamping.
const NORM: f32 = 1.0 / 100.0;

/// Per-column importance weights, cycled `latency, cpu, mem, err_rate,
/// latency, cpu, mem, err_rate, ...` across the flattened input.
const WEIGHTS: [f32; 4] = [0.4, 0.3, 0.2, 0.1];

/// Naive scalar scorer: normalize → clamp → weighted-sum, one element at a
/// time. Given, complete — this is your baseline to profile (`cargo
/// flamegraph --bench score`, see the README) and the reference
/// implementation `score_simd` must match within 1e-4 relative tolerance.
pub fn score_scalar(values: &[f32]) -> f32 {
    let mut total = 0.0f32;
    for (i, &v) in values.iter().enumerate() {
        let normalized = v * NORM;
        let clamped = normalized.clamp(0.0, 1.0);
        total += clamped * WEIGHTS[i % WEIGHTS.len()];
    }
    total
}

/// TODO(lab8): vectorize `score_scalar` with `std::simd::f32x8`.
///
/// Hints:
/// - Add `#![feature(portable_simd)]` at the top of this file and
///   `use std::simd::prelude::*;`.
/// - Process the input 8 floats at a time; handle whatever's left over
///   (0..7 elements) with a scalar loop — same idea as `chunks_exact` /
///   `as_chunks`.
/// - `WEIGHTS` has period 4 and your SIMD chunks are 8 lanes wide — think
///   about whether that lets you build one constant lane-weight vector
///   that's correct for every chunk, instead of gathering per lane.
/// - The scalar tail needs to keep using the *global* index (not restart
///   from 0) so its weight lines up with what `score_scalar` would compute.
pub fn score_simd(values: &[f32]) -> f32 {
    todo!("lab8: implement with std::simd::f32x8 + a scalar tail")
}

/// TODO(lab8): a single-threaded bump ("arena") allocator.
///
/// Public API to implement:
/// - `Arena::new(capacity: usize) -> Self`
/// - `Arena::alloc<T>(&self, val: T) -> &T` — note `&self`, not `&mut
///   self`. Think about why that signature is desirable (can you allocate
///   two objects from the same arena "at the same time" with `&mut self`?),
///   and what kind of interior mutability lets you bump a cursor through a
///   shared reference. Also think about what that implies for `reset`
///   below and the borrow checker.
/// - `Arena::reset(&mut self)` — reclaims all space so it can be reused.
///
/// Hints:
/// - `std::alloc::{alloc, dealloc, Layout}` to own a fixed backing buffer;
///   don't forget a matching `Drop` impl.
/// - `std::cell::Cell<usize>` for the bump cursor.
/// - `Layout::new::<T>()` gives you `T`'s size and alignment; use
///   `pointer::align_offset` (or manual bit math) to insert the right
///   amount of padding before each allocation.
/// - Panic (don't silently corrupt memory) if a requested allocation
///   doesn't fit in the remaining capacity.
/// - Every `unsafe` block needs a `// SAFETY:` comment justifying it.
///
/// See the lab README's "Steps" section for the full guided walkthrough,
/// including the `parse_batch`-style use case this is meant to speed up.
pub struct Arena {
    // TODO(lab8): choose your fields.
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        todo!("lab8: allocate a `capacity`-byte buffer and set up the bump cursor")
    }

    pub fn alloc<T>(&self, val: T) -> &T {
        todo!("lab8: bump-allocate space for `val`, aligned for `T`, and write it in")
    }

    pub fn reset(&mut self) {
        todo!("lab8: rewind the bump cursor so space can be reused")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `score_scalar` is given/complete, so this passes out of the box.
    // As you implement `score_simd` and `Arena`, add your own tests here —
    // the required test names
    // (`test_simd_matches_scalar_epsilon`, `test_arena_alignment`,
    // `test_arena_reset_reuses`) are a good target to aim for.
    #[test]
    fn score_scalar_matches_hand_computed_value() {
        // len 1 => only WEIGHTS[0] applies; 50.0 normalizes to 0.5.
        let v = score_scalar(&[50.0]);
        assert!((v - 0.5 * WEIGHTS[0]).abs() < 1e-6);
    }
}
