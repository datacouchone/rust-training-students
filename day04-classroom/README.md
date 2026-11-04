# Day 4: four-hour afternoon

25 min memory concepts; 30 min Lab 8B; 25 min SIMD concepts; 40 min Lab 8C;
10 min break; 10 min final briefing; 90 min final challenge; 10 min review.

## One evolving collector — slides 17–22

File: `demos/8b/src/lib.rs`. Start at STAGE 0. The instructor adds each step:

0. Store path bytes per row (baseline; not yet the complete contract).
1. Share path ranges through a hash dictionary. Forced collisions intentionally expose the missing equality check.
2. Compare actual path bytes inside collision buckets.
3. Validate limits and overflow before mutation; rejection preserves state.
4. Advance the epoch on reset; old tickets are rejected. Tickets belong to one originating Batch.
5. Retain small allocations, discard oversized capacities. Now run the full original acceptance suite.

```sh
python3 day04-classroom/run.py memory
```

Run commands from the participant repository root. The runner checks the SAVED
stage, measures it against the independent owned-string baseline, saves raw CSV
under `day04-classroom/results/memory`, and compares matching previous-stage
observations. Seven samples, same fixture sizes; comparisons between stages
are sequential measurements, not simultaneous controlled experiments. Construction,
ingestion and reset are included. Requested allocation bytes are not RSS.

## One evolving query — slides 24–30

File: `demos/8c/src/lib.rs`. `scan_simd` is the evolving candidate slot; stages
0–2 still use scalar code. `scan_scalar` stays the independent oracle.

0. Materialize matching latencies, then aggregate; assumes valid input shape.
1. Add input/query validation.
2. Fuse predicate and aggregation; remove the temporary Vec.
3. Introduce eight-lane SIMD and the full predicate mask. This teaching version
   still has u32 lane sums and omits tails: only the bounded, multiple-of-eight
   comparison fixture is supported. It is not a complete implementation.
4. Widen before reducing. Tails are still the next step.
5. Add tails using the global row index. Run the full original acceptance suite.
6. Select a complete candidate in scan_auto; scalar is a valid choice. Compare
   all implementations before inventing a size threshold.

```sh
python3 day04-classroom/run.py scan
```

The main comparison uses the same 128/4096-row, bounded-latency fixtures at every
stage, with dense and sparse validity. It reports ns per scan and saves raw CSV.
Correctness checks separately challenge shapes, wide sums and all tail lengths
as those concepts are added. Portable SIMD uses the installed pinned nightly.
These columns illustrate Arrow concepts but are not Arrow IPC. The original
Arrow IPC workbench remains optional follow-up material.

## Focused practice — 8B 30 minutes; 8C 40 minutes

8B: `labs/8b/src/lib.rs`. push/get are supplied. Implement reset without changing
its signature or the tests. Check epoch exhaustion before mutation, clear logical
state, and apply all three retention thresholds.

8C: [build the SIMD kernel](labs/8c/README.md), 40 minutes. Implement the predicate
mask, widened reduction and scalar tail. Validation and loads are supplied.
Measure your lab implementation, not the instructor demo. Read [SIMD-GUIDE.md](SIMD-GUIDE.md).

```sh
cargo +nightly-2026-07-22 test --manifest-path day04-classroom/labs/8b/Cargo.toml --offline --locked
cargo +nightly-2026-07-22 test --manifest-path day04-classroom/labs/8c/Cargo.toml --offline --locked
```

8B: 5 minutes inspect; 20 minutes implement/check/measure; 5 minutes explain.
8C: follow its 40-minute implementation sequence.
The original full production-labs/8B and 8C attempts remain unchanged.

## Final challenge — 90 minutes in pairs

Use [the SIMD telemetry final](final/README.md). Non-SIMD service repairs are supplied;
run their examples and interpret outcomes briefly. Implement and measure SIMD on
Event records, preserve the scalar oracle, then run healthy-simd integration.

```sh
python3 day04-classroom/final/run.py baseline
python3 day04-classroom/final/run.py simd-test
python3 day04-classroom/final/run.py acceptance
python3 day04-classroom/final/run.py simd-bench
python3 day04-classroom/final/run.py healthy-simd
```

The original full gateway challenge and original production labs remain available.


Timing: median nanoseconds per scan across seven samples. Lower is better.

| Workload | Stage 2: scalar candidate | Stage 3: SIMD candidate | Time reduction |
|---|---:|---:|---:|
| 128 rows, all valid | 125 ns | 58 ns               | 53.6% |
| 128 rows, sparse validity | 75 ns | 58 ns          | 22.7% |
| 4,096 rows, all valid | 2,678 ns | 1,971 ns        | 26.4% |
| 4,096 rows, sparse validity | 2,673 ns | 2,050 ns  | 23.3% |

Correctness: both stages returned the same results.

| Workload | Matched rows, both stages | Total latency, both stages |
|---|---:|---:|
| 128 rows, all valid | 3 | 154,149 |
| 128 rows, sparse validity | 0 | 0 |
| 4,096 rows, all valid | 107 | 8,002,422 |
| 4,096 rows, sparse validity | 26 | 1,941,732 |
