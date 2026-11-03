# Day 4 — tune real service components

Nine worked demonstrations accompany M8's three participant labs. These use
ArcSwap, serde_json, Apache Arrow and Criterion. They are separate from the lab
starters: running or replacing a demo does not complete anyone's assignment.

The engineering question is **which implementation should this workload use?**
Correctness is a gate. Throughput, allocation lifetime, update semantics and
maintenance cost decide whether the change earns its complexity.

## Problems, implementation choices and course mapping

| Worked examples | Service problem and implementation | Day 4 lab connection |
|---|---|---|
| `routing_owned`, `routing_reload`, `routing_guard` | A read-mostly gateway configuration changes while requests are active. Compare RwLock + Arc, owned ArcSwap reads and short guards. Compile and validate before publication; pin one generation for a whole decision; reject invalid reloads without losing the last good snapshot. | **8A:** snapshot ownership, separating build cost from request cost, comparing equivalent decisions. Demo matches exact normalized hosts; the assignment implements tenant-aware longest segment-prefix routing. |
| `encode_reuse`, `encode_retention`, `encode_quota` | Telemetry workers repeatedly serialize NDJSON. Use the real serde_json writer, reuse worker-owned buffers, discard burst capacity after flush, and reject oversized output without exposing a partial batch. Follow 100 normal batches, one burst and 100 quiet batches. | **8B:** allocation churn versus retained capacity, bounded state, rejected updates and lifecycle policy. Demo controls output buffers; assignment deduplicates input path storage with collision-safe equality. |
| `arrow_nulls`, `arrow_fused`, `arrow_ipc` | Analytics receives nullable Arrow RecordBatches. Compare library mask/filter/cast/sum kernels with a specialized fused aggregate; preserve independent nulls, status bounds and widened sums. Round-trip sliced arrays through real Arrow IPC. | **8C:** fair scalar baseline, validity masks, widening, offsets/tails and workload-dependent dispatch. Demo evaluates existing kernels before the assignment explores explicit portable SIMD. |

Each demo has one marked `main` body. Coach **Show code → Run → Replace → Run**
compares its two variants; **Restore default code** restores only that demo.
All default variants run successfully. Replacements are experiments, not assumed
improvements. Core implementations are in [routing.rs](src/routing.rs),
[encoding.rs](src/encoding.rs) and [analytics.rs](src/analytics.rs).

## Run

From the student repository root, fetch this separate crate's dependencies once
before class. The existing production-labs workspace and lockfile are unchanged.

```sh
cargo +stable fetch --manifest-path day04-production/Cargo.toml --locked
cargo +stable test --manifest-path day04-production/Cargo.toml --offline --locked
cargo +stable run --release --manifest-path day04-production/Cargo.toml --bin routing_reload --offline --locked
cargo +stable run --release --manifest-path day04-production/Cargo.toml --bin encode_retention --offline --locked
cargo +stable run --release --manifest-path day04-production/Cargo.toml --bin arrow_fused --offline --locked
cargo +stable bench --manifest-path day04-production/Cargo.toml --bench service_paths --offline --locked -- --noplot
```

To measure one group, put `configuration_read`, `ndjson_steady_state`, or
`arrow_aggregate` before `--noplot`. Run benchmarks on an otherwise quiet machine.
Record CPU, compiler, release flags, workload and raw results. Criterion output
is under `day04-production/target/criterion/`. The lockfile fixes dependency
versions; preparation used Rust 1.97.1 on an Apple M2, arm64. No Docker or remote
service is required. A cold dependency build takes longer than a classroom run.

## What the measurements establish

- Criterion runs optimized code with warmup and 30 samples. Fixture creation,
  printing and correctness assertions stay outside timed loops. Return values
  and query inputs cross `black_box` boundaries.
- Configuration timings are **single-thread read cost**, not contended p99 or
  HTTP latency. A separate four-reader reload regression checks consistent
  generations. Old snapshots remain alive while requests own them. Frequent
  reloads plus slow requests can increase retained memory.
- JSON timings include actual serialization. One encoder survives many timed
  iterations. The retention demo measures capacity after reset across a burst;
  it does not estimate process RSS. The benchmark's steady-state threshold is
  512 KiB; the burst demo intentionally uses 16 KiB. Thresholds are inputs to
  a workload decision, not universal defaults.
- The logical output quota limits bytes appended, not allocator rounding,
  input event memory or the total across workers. The synchronous consumer
  must finish using borrowed output before reuse. For async export, transfer
  ownership or maintain a bounded buffer pool and bound queued batches.
- Arrow timings exclude ingestion and IPC. Small/large batches, null density
  and selection distribution are separate cases. A fused loop does not imply
  SIMD. The prepared results in [MEASUREMENTS.md](MEASUREMENTS.md) contain both
  wins and losses; do not set a universal dispatch threshold from one machine.
- Inputs are deterministic generated workloads, **not captured customer data**.
  Replay a sanitized representative distribution before making a service
  deployment decision. The components are real implementations, not a complete
  proxy, exporter or analytics service.

## Acceptance evidence and next engineering work

The regression suite checks invalid reload rollback, old snapshot ownership,
concurrent readers, JSON byte equivalence and escaping, exact quota boundaries,
recovery after rejection, burst/quiet capacity, explicit null/overflow outcomes,
all slice offsets 0–8, tail sizes and Arrow IPC round trips. Passing these tests
is a gate, not proof of all concurrent executions or a production certification.

Bring a result to review: workload and baseline; cost identified; preserved
contract; measured win and loss; configuration/release decision; rollback
metric. For 8A, include reload frequency and build amortization. For 8B, include
worker count and burst recurrence. For 8C, include result selectivity and whether
other operators reuse intermediate arrays. Keep raw samples, not only ratios.

## Industry sources and exact limits of the mapping

[Cloudflare's Pingora account](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/)
describes proxy architecture and connection reuse. It motivates examining shared
request-path costs; our exact-host snapshot implementation is an original
adaptation, not Pingora's routing implementation or a recreation of its speedup.

[Discord's Read States account](https://discord.com/blog/why-discord-is-switching-from-go-to-rust)
connects workload, memory management and latency investigation. Our NDJSON worker
is an original adaptation of the allocation-lifecycle problem, not Discord code.

[ArcSwap's documented use cases](https://docs.rs/arc-swap/1.9.3/arc_swap/) include
read-mostly configuration and routing snapshots.
[serde_json::to_writer](https://docs.rs/serde_json/latest/serde_json/fn.to_writer.html)
provides the actual serialization path.
[Arrow's columnar format](https://arrow.apache.org/docs/format/Columnar.html) and
[Arrow Rust compute kernels](https://docs.rs/arrow/56.2.1/arrow/compute/index.html)
define the representation and operations used here.
[Criterion](https://bheisler.github.io/criterion.rs/book/) provides the benchmark
harness. Sources reviewed 2026-11-02.
