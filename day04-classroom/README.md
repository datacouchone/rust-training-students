# Day 4: four-hour afternoon

25 min memory concepts; 30 min Lab 8B; 35 min scan concepts; 30 min Lab 8C;
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

## Focused practice — 30 minutes each

8B: `labs/8b/src/lib.rs`. push/get are supplied. Implement reset without changing
its signature or the tests. Check epoch exhaustion before mutation, clear logical
state, and apply all three retention thresholds.

8C: `labs/8c/src/lib.rs`. Scalar/unrolled/SIMD are supplied. Measure the completed
demo, implement scan_auto, run all tests, and write a short evidence-based choice.
There is no required speedup and no prescribed cutoff.

```sh
cargo +nightly-2026-07-22 test --manifest-path day04-classroom/labs/8b/Cargo.toml --offline --locked
cargo +nightly-2026-07-22 test --manifest-path day04-classroom/labs/8c/Cargo.toml --offline --locked
```

5 minutes inspect; 20 minutes implement/check/measure; 5 minutes explain.
The original full production-labs/8B and 8C attempts remain unchanged.

## Final challenge — 90 minutes in pairs

Use `day04-classroom/final`, not the original challenge. Admission, retry-budget
accounting, route indexing and telemetry aggregation are supplied. Three repair
areas remain: policy lock scope; request deadline and task ownership; buffer
retention. Keep every original acceptance assertion, transport and metric.

```sh
python3 day04-classroom/final/run.py baseline
python3 day04-classroom/final/run.py acceptance
python3 day04-classroom/final/run.py healthy
python3 day04-classroom/final/run.py slow
python3 day04-classroom/final/run.py burst
```

15 minutes investigation, 50 minutes repairs, 25 minutes full checks and comparable
before/after measurements. Report successful outcomes as well as timing. Correct
code with an honest explanation matters more than a large speedup.
