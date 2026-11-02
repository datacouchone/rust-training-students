# 8C — Decide whether SIMD belongs in a columnar scan

Day 4 · M8 · independent production-use-case lab

An observability engine filters latency and status columns with nullable rows. Implement a competitive scalar scan and a SIMD candidate, preserve the nullable-data contract, then decide which implementation should serve each workload.

[Apache Arrow's columnar format](https://arrow.apache.org/docs/format/Columnar.html) motivates contiguous values and validity bitmaps. This small teaching format is not Arrow IPC-compatible. [Rust portable SIMD](https://doc.rust-lang.org/core/simd/) is experimental and pinned to the supplied nightly.

## Contract and difficult cases

Implement `scan_unrolled`, `scan_simd` and `scan_auto`. Preserve the independent `scan_scalar` oracle. Columns have equal length; bitmap length is exactly ceil(rows/8), with least-significant-bit-first validity. Ignore trailing bitmap padding bits. Validate the query status threshold in 100..599. Match only valid rows with latency at least the threshold and status between the query threshold and 599 inclusive. Invalid stored status codes never match.

Return exact matched count and a checked u64 latency total. Widen selected u32 lanes before reduction. Every tail length, empty input, offset slices and null distribution must work without assuming vector alignment. The global row index determines tail validity.

## Investigation sequence

1. Capture the scalar baseline; predict the effect of tiny inputs, sparse validity and threshold selectivity.
2. Implement a four-accumulator scalar candidate and run the same acceptance scenarios as SIMD.
3. Implement an eight-lane SIMD candidate with correct masks, widening and tail handling. Run all acceptance tests before timing.
4. Compare scalar, unrolled, SIMD and auto across 7, 128, 4099 and the supplied large length, with dense and sparse validity. The harness rotates execution order and runs repeated scans.
5. Choose dispatch from measurements, then validate on a held-out size and altered query selectivity. Always selecting a scalar implementation is a valid measured decision. No speedup threshold is graded.

A size-based SIMD cutoff is a hypothesis to test, not a required answer. The trainer reference selects unrolled scalar based on its local measurements; other hardware and query distributions can justify a different choice. Report total elapsed time **and iteration count**, or normalize per scan. Portable vector width does not guarantee a particular machine instruction. This is an in-memory scan experiment, not end-to-end request latency. Input generation and equivalence checks are outside timing; compiler optimization of scalar code is part of a fair comparison.

## Advanced transfer

Inspect generated assembly or add a different lane width and selectivity workload. Explain where CPU instructions, memory traffic or branch behavior account for results. Introduce no unchecked loads without an explicit safety argument and boundary tests.

## Commands

From `production-labs/`, before editing:

```sh
cargo +nightly-2026-07-22 test -p case08c-simd --test baseline --offline --locked
cargo +nightly-2026-07-22 run --release -p case08c-simd --bin measure-simd --offline --locked -- baseline 100003 > baseline.csv
```

After implementation:

```sh
cargo +nightly-2026-07-22 test -p case08c-simd --offline --locked
cargo +nightly-2026-07-22 run --release -p case08c-simd --bin measure-simd --offline --locked -- compare 100003 > compare.csv
```

Acceptance failures are expected in the starter. Keep APIs and assertions intact. Work one named test at a time with `cargo +nightly-2026-07-22 test -p case08c-simd --test acceptance TEST_NAME --offline --locked -- --exact`. Record your results in [EXPERIMENT.md](EXPERIMENT.md). Each lab runs independently and needs no Docker or external service.

Sources reviewed 2026-10-31. Industry accounts motivate these original adaptations; passing local tests does not certify production readiness.
