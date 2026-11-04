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

## Recorded classroom comparison — stage 2 → stage 3

Observed on the trainer machine on 2026-11-03. These are the cumulative demo's
results, not measurements of your unfinished lab implementation. Stage 2 is the
fused scalar candidate; stage 3 introduces eight-lane SIMD masks and reduction.

Median nanoseconds per scan across seven samples; each sample runs 500 scans.
Lower is better. Time reduction = (before − after) / before × 100%.

| Workload | Stage 2: scalar candidate | Stage 3: SIMD candidate | Time reduction |
|---|---:|---:|---:|
| 128 rows, all valid | 125 ns | 58 ns | **53.6%** |
| 128 rows, sparse validity | 75 ns | 58 ns | **22.7%** |
| 4,096 rows, all valid | 2,678 ns | 1,971 ns | **26.4%** |
| 4,096 rows, sparse validity | 2,673 ns | 2,050 ns | **23.3%** |

Both stages returned the same answers for these workloads:

| Workload | Matched rows, both stages | Total latency, both stages |
|---|---:|---:|
| 128 rows, all valid | 3 | 154,149 |
| 128 rows, sparse validity | 0 | 0 |
| 4,096 rows, all valid | 107 | 8,002,422 |
| 4,096 rows, sparse validity | 26 | 1,941,732 |

The SIMD candidate produced the same counts and totals and took less time in all
four measured cases. Even the zero-match case must inspect the input to determine
that no rows qualify.

**Interpretation:** these were separate runs. The unchanged scalar reference also
moved: 125 → 85 ns for 128 all-valid rows. Therefore, the whole 53.6% reduction
cannot confidently be attributed to the code change alone. Repeat comparisons
before claiming a reliable speedup; these are not universal hardware results.

**Stage 3 is incomplete:** these timing fixtures use bounded values and lengths
divisible by eight. They do not establish correctness for wide sums or tails.
Stages 4 and 5 add those repairs. Keep all correctness checks when measuring your
completed lab.

Source snapshots: stage 2 saved 2026-11-03 16:15:55, stage 3 saved 2026-11-03 16:19:39 (trainer local time, Europe/Brussels). Raw demo CSV paths: `day04-classroom/results/scan/stage-2.csv` and `stage-3.csv`; generated results are local and not included in Git.
