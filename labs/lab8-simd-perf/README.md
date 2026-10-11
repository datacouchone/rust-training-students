# Lab 8 — SIMD + Arena Allocation + Profiling

Module 8: Profiling, Flamegraphs, `std::simd`, and Custom Allocation.

## Goal

Take a CPU-bound "log-metrics scorer" from naive-scalar to vectorized, and
practice the workflow that gets you there in real code: **profile first,
then optimize, then profile again to check you actually won.**

1. **`score_simd`** — vectorize the given `score_scalar` baseline with
   `std::simd::f32x8` (`#![feature(portable_simd)]`, nightly-only), with a
   correct scalar tail for lengths that aren't a multiple of 8.
2. **`Arena`** — a hand-rolled bump allocator (`alloc<T>(&self, val: T) ->
   &T`, `reset(&mut self)`), used in the solution's `parse_batch` to turn
   repeated per-batch heap-allocation churn into a handful of pointer
   bumps reclaimed in one `O(1)` reset.
3. **Profiling** — generate and read a flamegraph over the Criterion bench
   for both versions, and produce an honest, measured speedup number
   instead of quoting a textbook multiplier.

By the end you should be able to look at a hot, branchy numeric loop and
reason concretely about: what a flamegraph tells you that a stopwatch
doesn't, what `std::simd` buys you and what it costs to set up correctly
(alignment, tails, lane-shuffling weights), and why handing out `&T` from
`&self` in a custom allocator is a deliberate, checked-by-the-compiler
design choice, not a shortcut around `&mut`.

## Time box

**90–120 minutes.** Roughly: 10 min reading the starter + this README and
running a baseline flamegraph on the scalar bench, 30–40 min on
`score_simd` (the weight-cycling-across-lanes part is the fiddly bit —
don't rush it), 30–40 min on `Arena` (interior mutability + alignment is
the fiddly bit there), 15–20 min re-profiling and comparing scalar vs.
SIMD flamegraphs, remaining time for stretch goals.

## Starter entry point

`starter/src/lib.rs`. `score_scalar` is **given and complete** — it's the
naive baseline (per-element normalize → clamp → weighted-sum, with a
`i % WEIGHTS.len()` index recomputed every iteration) that you'll profile
and then beat. `score_simd` and `Arena` (`new`, `alloc`, `reset`) are
`todo!()`.

The starter also has its own `benches/score.rs` (Criterion, `harness =
false`), so you can flamegraph your own progress the whole way through —
it compiles from the start, since a `todo!()` body only panics if the
`simd` half of the bench actually *runs*.

If you get stuck, `solution/src/lib.rs` is the answer key, and its doc
comments spell out the design reasoning in full (especially the "why
`&self`" note on `Arena`) — but working through the reasoning yourself
first is the point of the lab.

## Steps

1. `cd "labs"`, then `source "$HOME/.cargo/env"` and
   `export CARGO_TARGET_DIR="${PWD}/target/lab8"`
   (or your own scratch target dir) so builds don't collide with other labs.
2. `cargo test -p lab8-simd-perf-starter` — confirm the one given test
   (`score_scalar` smoke test) passes.
3. **Baseline profile.** Install flamegraph if you haven't:
   `cargo install flamegraph`. Then, from `labs/`:

   ```bash
   cargo flamegraph --bench score -p lab8-simd-perf-starter -- --bench
   ```

   The trailing `-- --bench` matters: without it, flamegraph's own
   argument handling and Criterion's CLI parsing can talk past each other
   and Criterion runs nothing. (On macOS check `xcrun xctrace version`
   and profiler permissions before class; full Xcode may be required.
   See the current [tool documentation](https://github.com/flamegraph-rs/flamegraph). On Linux it
   uses `perf`, which needs `/proc/sys/kernel/perf_event_paranoid <= 1` or
   `sudo`.) Open the generated `flamegraph.svg` — with `score_simd` still
   `todo!()`, every sample should land in `score_scalar`'s loop. Note
   which lines are hot (the `WEIGHTS[i % WEIGHTS.len()]` index and the
   `max`/`min`/`clamp` sequence are the usual suspects) — that's your
   vectorization target.
4. **Implement `score_simd`.** Process the input 8 floats at a time with
   `f32x8`; handle the 0–7 leftover elements with a scalar loop
   (`slice::as_chunks::<8>()` gives you `(&[[f32; 8]], &[f32])` for exactly
   this split). Work out why a single constant lane-weight vector
   `[w0, w1, w2, w3, w0, w1, w2, w3]` is correct for *every* full chunk
   (hint: `WEIGHTS.len() == 4`, chunk width `== 8` — what's `chunk_start %
   4` always equal to?), and make sure your scalar tail keeps using the
   *global* index so its weight lines up with what `score_scalar` would
   compute for the same elements, not a fresh `0, 1, 2, ...` count.
5. Write `test_simd_matches_scalar_epsilon`: a tiny hand-rolled LCG (no
   `rand` dependency — a `u64` state, multiply-add, take the high bits) for
   deterministic pseudo-random `f32` inputs, generate values *outside*
   `[0, 100]` on both ends so the clamp branch is actually exercised, and
   assert `score_scalar` and `score_simd` agree within 1e-4 *relative*
   tolerance. Cover awkward lengths on both sides of the 8-lane boundary:
   at minimum `0, 1, 7, 8, 9`, plus a large length with a nonzero remainder
   (e.g. `1_000_003`) so the tail loop is exercised at scale, not just at
   the small end.
6. `cargo test -p lab8-simd-perf-starter` until that test is green.
7. **Implement `Arena`.** Before writing any code, work out on paper: what
   field(s) do you need to own a fixed backing buffer
   (`std::alloc::{alloc, dealloc, Layout}`, freed in `Drop`)? What field
   lets `alloc(&self, ..)` bump a cursor while only borrowing `self`
   immutably (`std::cell::Cell<usize>`)? For each `alloc::<T>()` call, use
   `Layout::new::<T>()` and `pointer::align_offset` to insert the right
   amount of padding *before* writing `T`, and panic (don't silently
   corrupt memory) if the arena doesn't have room. Every `unsafe` block
   needs a `// SAFETY:` comment stating the actual invariant that makes it
   sound, not a restatement of the line.
8. Write `test_arena_alignment` (interleave allocations of different
   `T`s — `u8`, `u16`, `u64`, `u128`, and something with `#[repr(align(32))]`
   — and check every returned pointer's address is a multiple of that
   type's alignment) and `test_arena_reset_reuses` (a strong version:
   size the arena for *exactly one* allocation, loop `alloc` → assert
   value and address → `reset` a few hundred times, and assert the address
   never moves — if `reset` didn't actually reclaim the space, the second
   iteration would panic with "out of memory", not just be slow).
9. `cargo test -p lab8-simd-perf-starter` until all tests are green, then
   `cargo clippy -p lab8-simd-perf-starter --all-targets` — clean.
10. **Re-profile and compare.**

    ```bash
    cargo flamegraph --bench score -p lab8-simd-perf-starter -- --bench
    ```

    Compare the new flamegraph to your baseline from step 3: the `simd`
    frames should be a much smaller fraction of total samples than
    `scalar`'s were, and the SIMD leaf frames should be dominated by
    vector load/fma/reduce instructions rather than a per-element modulo.
    Then run the actual numbers:

    ```bash
    cargo bench -p lab8-simd-perf-starter --bench score
    ```

    Criterion prints median times for `score/scalar/<n>` and
    `score/simd/<n>` at each input size — compute the ratio yourself. See
    "Profiling workflow" below for how to read what the flamegraph is
    (and isn't) telling you.

## Profiling workflow

`cargo flamegraph` runs a binary under a sampling profiler (`xctrace` on
macOS, `perf` on Linux) and turns the call-stack samples into an
interactive SVG: wider frames = more samples landed there = more wall-clock
time spent in that function (inclusive of its callees, stacked above it).
For this lab:

```bash
cargo install flamegraph            # one-time
cargo flamegraph --bench score -p lab8-simd-perf-solution -- --bench
```

- The `-- --bench` is passed through to the Criterion binary itself and
  tells it to actually run its benchmarks (Criterion's own CLI otherwise
  expects to be driven like a test harness). Without it you'll get a
  flamegraph of... Criterion parsing its own arguments and exiting.
- Criterion's default per-benchmark run is a few seconds, which is enough
  wall-clock time to get a meaningful number of samples across
  `score/scalar/*` and `score/simd/*` — you don't need to crank up
  `--measurement-time` just to get a flamegraph, only if the SVG looks
  too sparse to read.
- Read it top-down from `main`: you're looking for where the *width*
  concentrates. For `score_scalar`, expect most width in the loop body
  itself (index, multiply, clamp, index into `WEIGHTS`). For `score_simd`,
  expect the per-iteration frame to shrink dramatically and, at large
  input sizes, for a bigger *relative* share of the profile to shift
  toward memory access (the loop is doing 8x less arithmetic-dispatch
  work per element, so whatever's left is comparatively more visible).
- A flamegraph tells you *where* time goes, not *how much total* time a
  change saved — pair it with the Criterion numbers (`cargo bench`) for
  the actual before/after comparison, and use the flamegraph to explain
  *why* the numbers moved.

**Honest speedup framing:** don't expect (or promise) exactly `10x`. In
local testing on this repo's pinned toolchain (Apple M2, `aarch64`,
`f32x8` — i.e. two 128-bit-equivalent vector ops per "lane group", not a
true 256-bit AVX2 run), `score_simd` measured **roughly 6–7x** faster than
`score_scalar` across the 1K/64K/1M-element bench sizes. The realistic
range across different hardware, data sizes, and how aggressively you
hand-tune beyond the straightforward `f32x8` port is **4–10x** — CPUs with
wider native vectors (AVX2/AVX-512 on x86_64) or a more favorable
instructions-per-element ratio in the scalar baseline can land higher; a
short, cache-resident input, a scalar baseline the compiler auto-vectorizes
part of anyway, or memory-bandwidth-bound large inputs can land lower. The
curriculum's "10x" is the **stretch goal**, not the passing bar —
correctness (the epsilon test) and a real, measured improvement are the
done-criteria; treat double-digit speedup as a bonus you get to brag about
if your hardware and data size cooperate, not a number to force by
tweaking the benchmark until it appears.

## Done-criteria

- [ ] `cargo test -p lab8-simd-perf-solution` — all tests pass:
      `test_simd_matches_scalar_epsilon` (1e-4 relative tolerance, across
      awkward lengths including 0, 1, 7, 8, 9, and a large
      nonzero-remainder length), `test_arena_alignment`,
      `test_arena_reset_reuses`.
- [ ] `cargo bench -p lab8-simd-perf-solution --no-run` — the Criterion
      bench (`benches/score.rs`, scalar vs. simd) compiles. (You don't
      need to run the full bench to be "done" — see Steps 10 / Profiling
      workflow above for actually running it.)
- [ ] `cargo build -p lab8-simd-perf-starter` compiles (bodies may still
      be `todo!()`); `cargo build --workspace` still succeeds for every
      other lab.
- [ ] Every `unsafe` block in `Arena` has a `// SAFETY:` comment that
      states the actual invariant making it sound (bounds/alignment
      arguments tied to `self.offset`/`self.capacity`, not a restatement
      of the line).
- [ ] `cargo clippy -p lab8-simd-perf-solution --all-targets` is clean.
- [ ] You've generated at least one real flamegraph (scalar or SIMD) and
      can point at a specific frame and say what it tells you.
- [ ] You can explain, out loud: why does `Arena::alloc` take `&self`
      (not `&mut self`) yet still safely hand out a mutable-looking `&T`
      into freshly-written memory, and why does the borrow checker stop
      you from calling `reset` while an earlier `alloc`'d reference is
      still alive?

## Stretch goals

- Push past the straightforward `f32x8` port toward the "10x" territory:
  try unrolling the SIMD loop to process two `f32x8` vectors per
  iteration (reduces loop-overhead-to-work ratio further), or restructure
  `parse_batch` + scoring so the scorer reads records the arena already
  laid out contiguously (better cache behavior than reading through a
  `Vec<&LogRecord>` of scattered pointers). Re-measure with `cargo bench`
  after each change — does it actually move the number, or just add
  complexity?
- Add a second bench input pattern that's adversarial for SIMD: instead of
  `WEIGHTS`'s period-4 pattern lining up neatly with 8-wide chunks, try a
  period-3 (or otherwise chunk-width-coprime) weight cycle and work out
  what changes about how you'd have to build the lane-weight vector (hint:
  you can no longer use one constant vector for every chunk — what do you
  need instead, and what does that cost?).
- Extend `Arena` with an `alloc_slice_copy<T: Copy>(&self, vals: &[T]) ->
  &[T]` that bump-allocates room for a whole slice at once (one padding
  computation, one `copy_from_slice`, instead of `n` individual
  `alloc::<T>()` calls) and rewrite `parse_batch` to return `&[LogRecord]`
  instead of `Vec<&LogRecord>` — one arena allocation and zero `Vec`
  allocations per batch instead of one `Vec<&LogRecord>` allocation per
  batch. Benchmark the difference.
- Run `cargo miri test -p lab8-simd-perf-solution`. The arena tests
  (`test_arena_alignment`, `test_arena_reset_reuses`,
  `test_parse_batch_uses_arena`) pass cleanly — this is a genuinely useful
  check given how much raw-pointer arithmetic `Arena::alloc` does. Miri
  *does* run `std::simd` code on this toolchain, but interpretation is
  slow, and `test_simd_matches_scalar_epsilon`'s `1_000_003`-element case
  can take a very long time (interpreting ~1M loop iterations one at a
  time). For a quick Miri smoke check, temporarily shrink that test's
  `lengths` array to just `[0, 1, 7, 8, 9]` while iterating under Miri,
  then switch it back before calling the lab done.
- Make `Arena` generic over an alignment/capacity growth policy, or add a
  `Drop`-tracking variant (store a small `Vec<fn(*mut u8)>` of destructor
  thunks alongside each allocation) so `reset` *can* run `Drop` for types
  that need it — then discuss the trade-off: what did you give up (the
  `&self`-only, zero-bookkeeping simplicity) to get that back?
