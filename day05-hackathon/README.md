# Project Relay — paired production incident hackathon

**Day 5 · teams of exactly two · 150 minutes reserved.** Allow a further 15–30 minutes for cross-team demonstrations when the group is large. This is planned exercise time, not a measured promise that every pair will finish. The exercise has a core repair track and stretch work; no prize for finishing first.

## The incident

You own a multi-tenant telemetry ingestion gateway. A single request succeeds. After a client timeout, a retry may charge the same record twice. A cancelled partial read can lose framing progress. Some rejected records change the ledger. Enrichment work survives its caller, a traffic burst retains memory, and the release gate can approve untrustworthy evidence.

Your team must restore the contract, show the original trigger no longer breaks it, and decide whether a candidate is safe to release. A green test suite alone is not a release plan.

Industry mapping: [Amazon's idempotent API contracts](https://docs.aws.amazon.com/ec2/latest/devguide/ec2-api-idempotency.html) distinguish a retry with the same identity and parameters from a conflicting request. [Amazon's rollback-safety practice](https://aws.amazon.com/builders-library/ensuring-rollback-safety-during-deployments/) asks whether old and new versions remain compatible. [Tokio I/O](https://docs.rs/tokio/1.53.1/tokio/io/trait.AsyncReadExt.html) supplies the cancellation contracts we use. This is an original synthetic service, not Amazon source or a reproduction of an outage.

## Start from the participant repository root

```sh
python3 day05-hackathon/run.py baseline
python3 day05-hackathon/run.py probe load
python3 day05-hackathon/run.py probe replay
python3 day05-hackathon/run.py acceptance
```

The smoke test passes before repair. Acceptance failures are intentional. First-time setup, if dependencies are not cached:

```sh
cargo +nightly-2026-07-22 fetch --manifest-path day05-hackathon/Cargo.toml --locked
```

The runner uses the pinned nightly, offline/locked builds and a watchdog. Supported native platforms: macOS/Linux with the system C runtime. No Docker, remote endpoint or C compiler is required. Do not share `coach/` or copy a reference solution into your submission.

## The service boundary

`client → bounded frame → Untrusted → Validated → admission → enrichment wait → ledger commit → ACK`

The load/replay/slow scenarios use actual TCP on **127.0.0.1 and ephemeral ports**, a two-worker Tokio runtime, at most 48 load clients and 64 connection handlers. Enrichment is a controlled async delay, not internet latency. The fragment probe and byte-boundary tests use in-memory duplex I/O for repeatable cancellation; they are explicitly not network timing measurements.

The receipt ledger is bounded, in memory and valid only for this process lifetime. It rejects new identities when full and still recognizes an existing replay. Do not silently evict receipts: that weakens duplicate protection. Durable deduplication, receipt expiry, crash recovery, authentication, TLS and multi-node coordination are out of scope. Document them in the release review. A local cancellation cannot undo a previously committed external side effect.

## Required contracts and editable files

| Station | File | Required behavior |
|---|---|---|
| A · Stream ownership | `src/codec.rs` | Retain consumed bytes across cancelled reads; distinguish empty/EOF/truncated; reject oversize before body allocation; poison terminal errors. |
| B · Trust boundary | `src/boundary.rs` | Safe borrowed C wrapper compares whole byte strings, including length. Validate version 1, a 21-byte explicit header, UTF-8 absolute path, <=128 path bytes. Only `Validated` reaches the ledger. |
| C · Transaction and replay | `src/ledger.rs` | `(tenant, request ID)` scopes identity. Identical body returns Replay without charging; changed body conflicts. Check arithmetic and both cardinalities before mutation. Rejection changes neither account nor receipts. |
| D · Async lifetime | `src/worker.rs` | One deadline includes admission, enrichment and lock waiting. No owned work remains after timeout/parent cancellation. Admission permits remain associated with that work. Independent enrichment waits overlap; commit is serialized. |
| E · Performance evidence | `src/analytics.rs` | Valid rows, latency >= threshold and status 500..=599 define both count and widened total. Avoid materializing selected rows. Release burst capacity above `retain`; keep small capacity. |
| F · Release gate | `src/rollout.rs` | Inconsistent/empty evidence means Hold. Duplicate effects mean Rollback. Otherwise <1,000 observations means Hold. At sufficient observations, >1% errors or p95 >50 ms means Rollback; else Promote. These are exercise thresholds, not universal SLOs. |

Keep `src/fixture.rs`, `src/main.rs`, existing tests and thresholds unchanged. Add your own regression tests. `pub` fields in the ledger support inspection for this exercise; discuss stronger encapsulation as a design improvement. The `memcmp` wrapper is memory-safe even in the faulty starter: the bug is comparing only a common prefix. Explain the unsafe preconditions; do not introduce deliberate UB. Do not serialize `NativeSummary` by reading padding. Values are decoded from explicit byte slices.

## Two-person workflow — 150 minutes

| Elapsed | Pair action | Evidence checkpoint |
|---|---|---|
| 00–15 | A drives probes; B records competing hypotheses. | Baseline + two diagnoses, one likely and one alternative. |
| 15–50 | A implements the first boundary/transaction repair; B writes/reviews its regression. | Previously failing trigger, exact contract and focused diff. |
| 50–85 | **Swap roles.** B drives stream/lifetime repair; A reviews ownership and cancellation. | Demonstrate cleanup and concurrency, not only happy-path results. |
| 85–115 | Pair on memory/query and performance. Repeat the same workload/settings. | Raw results, successful outcomes, p50/p95, active work, retained capacity. |
| 115–135 | Repair the release gate and challenge your deployment plan. | Canary window, stop signal, rollback owner and compatibility argument. |
| 135–150 | Exchange reviewer roles and prepare the incident handoff. | Five-minute demo + a decision both partners can defend. |

Review [EVENT-CARDS.md](EVENT-CARDS.md) at the indicated checkpoints. Fill [INCIDENT.md](INCIDENT.md). If blocked, use the staged [HINTS.md](HINTS.md); record the hint, then continue. Core requirements remain the same after a hint.

## Probe menu

```sh
python3 day05-hackathon/run.py probe fragment
python3 day05-hackathon/run.py probe layout
python3 day05-hackathon/run.py probe replay
python3 day05-hackathon/run.py probe load --clients 24
python3 day05-hackathon/run.py probe slow --clients 12
python3 day05-hackathon/run.py probe burst
python3 day05-hackathon/run.py probe query
python3 day05-hackathon/run.py probe rollout
```

Load output labels applied/replay/rejected/timeout/transport errors separately. Throughput counts completed responses, including failures. p95 comes from a small closed-loop batch: repeat trials, keep offered workload constant and avoid SLO claims. Run one benchmark at a time. The slow scenario reports both active work at response completion and state after settling; this exposes effects that outlive the response. Native FFI layout is observed on your machine, not a portable packet format.

## How this brings Days 1–5 together

| Day | Work you must demonstrate |
|---|---|
| 1 · ownership/layout | Explain owned decoder progress, permit/guard Drop, borrowed C slices and why padding is not wire data. |
| 2 · unsafe boundaries/invariants | Audit the actual `extern "C"` call; repair full equality; enforce Untrusted → Validated before commit. No invalid value can bypass the constructor through public fields. |
| 3 · concurrency/async | Explain what Arc shares, what the mutex serializes, what SeqCst counters measure, and who owns work while waiting, timing out or being cancelled. Atomics alone do not make a multi-field transaction atomic. |
| 4 · tuning/measurement | Preserve predicates and widened arithmetic; tune lock scope and retained allocation; compare matched workloads and reject misleading speedups. SIMD is optional and must beat the scalar oracle with equivalent results. |
| 5 · diagnosis/release | Reproduce byte fragmentation and lost ACK; add regressions; produce a correlated incident timeline and a canary/rollback decision. |

This integrates the central engineering responsibilities from each day, not every API taught. Your report must distinguish documented guarantees, observed results and unresolved limits.

## Submission and scoring

Submit: patch, original/current logs, all acceptance results, at least two new adversarial regressions, an unsafe-invariant review, performance comparison, incident report and release decision. Both partners explain one repair written by the other.

100 points: contracts/regressions 45; unsafe and ownership argument 10; measurement quality 20; canary/rollback reasoning 15; pair handoff 10. Unsafe memory access, deleted assertions, relaxed thresholds or rejecting all traffic cannot count as a successful repair. A defensible **Hold** can score better than an unsupported **Promote**. Private marking guidance stays with the trainer.

Stretch after core: bounded expiry with an explicit deduplication horizon; a persistent receipt design with crash boundaries; scalar/SIMD differential tests for offset/tail slices; generation-compatible rollout data. Implement and test a chosen extension only after writing its new contract. Do not claim a design sketch is implemented.
