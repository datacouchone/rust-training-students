# Preparation measurements — 2026-11-02

Apple M2 / arm64 / macOS / Rust 1.97.1. Release build with debug symbols; no target-cpu override. Criterion 0.5.1: 1 s warmup, 2 s measurement, 30 samples per case. Generated fixtures; one local run. These are mean estimates and their 95% bootstrap intervals, not service latency percentiles. Raw numerical data: [measurements.csv](measurements.csv).

| Workload | Mean (ns) | 95% interval (ns) |
|---|---:|---:|
| arrow_aggregate/fused/n128_null17_error19 | 213.4 | 212.8–214.0 |
| arrow_aggregate/fused/n65539_null17_error19 | 130907.1 | 130714.3–131108.0 |
| arrow_aggregate/fused/n65539_null2_error2 | 92987.8 | 92754.4–93285.7 |
| arrow_aggregate/fused/n65539_null97_error1 | 105416.2 | 105020.8–105961.8 |
| arrow_aggregate/kernels/n128_null17_error19 | 1119.4 | 1116.1–1123.6 |
| arrow_aggregate/kernels/n65539_null17_error19 | 37812.4 | 37730.1–37889.7 |
| arrow_aggregate/kernels/n65539_null2_error2 | 32557.4 | 32402.3–32756.6 |
| arrow_aggregate/kernels/n65539_null97_error1 | 133222.5 | 132912.9–133578.3 |
| configuration_read/arc_swap_guard | 18.0 | 17.9–18.1 |
| configuration_read/arc_swap_owned | 19.4 | 19.2–19.7 |
| configuration_read/rwlock_arc | 20.4 | 20.2–20.9 |
| ndjson_steady_state/Bounded/32 | 4654.8 | 4595.5–4760.7 |
| ndjson_steady_state/Bounded/4096 | 593443.6 | 592766.3–594143.1 |
| ndjson_steady_state/Fresh/32 | 5082.5 | 5073.6–5090.5 |
| ndjson_steady_state/Fresh/4096 | 603424.2 | 598348.3–612604.5 |
| ndjson_steady_state/Reuse/32 | 4623.2 | 4601.0–4650.0 |
| ndjson_steady_state/Reuse/4096 | 596290.8 | 593940.7–599071.6 |

Interpretation: the fused Arrow aggregate wins on the 128-row case but loses on the 65,539-row sparse-error case. Avoiding temporary arrays does not guarantee less CPU time. The dense-selection workload changes the outcome again. Inspect null/selection correlation: null2/error2 deliberately selects no valid rows.

The short ArcSwap guard has lower local read cost here. This does not establish contention scalability or an end-to-end request latency improvement. Reuse helps small NDJSON batches more than large ones in this run, where serialization dominates. The capacity-after-burst demo answers a different question from these steady-state timings.

Rerun on the deployment CPU and representative distributions; investigate noise/outliers and repeat selected comparisons before release. A workload-dependent dispatch policy needs a stable measured boundary and continued correctness equivalence, not a hard-coded threshold copied from this table.
