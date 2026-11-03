# 8C — Build the SIMD kernel (40 minutes)

Implement SIMD, not just dispatch. The scalar oracle, validation, vector loads and
unrolled comparison are supplied. Work in `src/lib.rs`; preserve every test.

1. 5 min: run baseline; predict the eight-lane example in [SIMD-GUIDE.md](../../SIMD-GUIDE.md).
2. 10 min: implement `predicate_mask` (validity AND latency AND both status bounds).
3. 10 min: implement `reduce_selected`: count true lanes, zero rejected lanes,
   widen each selected u32 to u64, then reduce. Casting after reduction is too late.
4. 5 min: finish the tail using global row indices and checked totals.
5. 10 min: all tests, benchmark YOUR lab, explain the result and any dispatch change.

From the participant repository root:

```sh
cargo +nightly-2026-07-22 test --manifest-path day04-classroom/labs/8c/Cargo.toml --offline --locked --test baseline
cargo +nightly-2026-07-22 test --manifest-path day04-classroom/labs/8c/Cargo.toml --offline --locked --test simd_steps
cargo +nightly-2026-07-22 test --manifest-path day04-classroom/labs/8c/Cargo.toml --offline --locked
cargo +nightly-2026-07-22 run --release --manifest-path day04-classroom/labs/8c/Cargo.toml --bin measure-simd --offline --locked -- compare 100003
```

The three `simd_steps` tests initially fail at intentional TODOs. Run one by adding
its exact name and `-- --exact`. Original acceptance tests cover all tails, offset
slices, nulls, padding bits, malformed input, wide values and differential cases.
`scan_auto` initially calls your SIMD kernel; choose another candidate only after
measuring. Do not replace `scan_simd` with the oracle to satisfy this exercise.

The benchmark measures scalar, unrolled, SIMD and auto at 7/128/4099/100003 rows,
dense/sparse validity and two status thresholds. Report elapsed_ns / iterations
as ns per scan, using the median of seven samples for each identical case.
Initialization and assertions are outside timing. There is no required speedup.

Extensions: implement a four-lane candidate; then a 16-lane candidate. Rework
validity extraction using global indices: the eight-lane shortcut `start/8`
plus `1 << lane` does not generalize. Test all tails before timing. Inspect the
assembly and explain one case where explicit SIMD loses. A no-op width change
without revisiting the bitmap mapping is not a valid solution.
