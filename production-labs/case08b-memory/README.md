# 8B — Bound telemetry memory after a traffic burst

Day 4 · M8 · independent production-use-case lab

A telemetry collector stores many repeated request paths. Per-row strings cause allocation churn, while clearing a large batch can keep excess capacity resident during quiet periods. Build bounded batch storage that deduplicates path bytes and applies an explicit capacity-retention policy after flush.

[Discord's Read States account](https://discord.com/blog/why-discord-is-switching-from-go-to-rust) motivates workload-specific allocation and latency investigation. The supplied collector is original teaching code, not Discord's implementation.

## Contract and difficult cases

Implement `Batch::push`, `get` and `reset`. Every row remains distinct even when paths share storage. Hash collisions must compare actual bytes. Paths are UTF-8 strings beginning with `/`. Enforce row, unique-path and logical payload-byte limits. Check aggregate overflow before mutating storage, counters or dictionary. Rejected insertion leaves state unchanged.

Tickets refer only to their originating Batch instance. Reset invalidates old tickets using a nonwrapping epoch; exhausted epoch returns an error without changing state. Emptying a batch resets all logical state. Reuse buffers within retention thresholds; discard oversized row, payload and dictionary buffers. Do not assert that `shrink_to_fit` guarantees an exact allocation size. The three retention fields use element, byte and bucket capacities respectively.

## Investigation sequence

1. Capture the supplied owned-string baseline for repeated paths, distinct paths and burst-then-idle workloads. Predict where interning helps and where it adds overhead.
2. Implement transactional insertion and collision-safe lookup. Start with `hash_collision_does_not_alias_paths` and `overflow_cannot_intern_a_rejected_path`.
3. Implement reset, stale-ticket rejection and capacity policy; run acceptance and the epoch-exhaustion unit test.
4. Compare allocation calls, peak requested bytes, retained requested bytes and elapsed time across all three workloads. Explain why fewer allocations can still mean slower execution or more retained memory.
5. Try held-out cardinality and batch size; recommend a retention policy with explicit tradeoffs.

The supplied allocation meter delegates to the system allocator and counts requested sizes. It does not measure RSS, allocator metadata, fragmentation or process-wide peak memory. Logical payload quotas do not cap total physical memory. The fixtures are bounded; the owned baseline's arithmetic is intended for these fixtures. Allocation failure handling and hostile-hash CPU bounds are out of scope. The timed region includes construction, ingestion and clearing/reset; input generation is excluded. Compare the differing reset policies explicitly.

## Advanced transfer

Add a multi-batch reuse experiment and a policy for occasional giant batches. Measure retention between batches; define a physical-memory budget model including row metadata and dictionary overhead. This extension requires new tests and measurements.

## Commands

From `production-labs/`, before editing:

```sh
cargo +nightly-2026-07-22 test -p case08b-memory --test baseline --offline --locked
cargo +nightly-2026-07-22 run --release -p case08b-memory --bin measure-memory --offline --locked -- baseline 5000 > baseline.csv
```

After implementation:

```sh
cargo +nightly-2026-07-22 test -p case08b-memory --offline --locked
cargo +nightly-2026-07-22 run --release -p case08b-memory --bin measure-memory --offline --locked -- compare 5000 > compare.csv
```

Acceptance failures are expected in the starter. Keep APIs and assertions intact. Work one named test at a time with `cargo +nightly-2026-07-22 test -p case08b-memory --test acceptance TEST_NAME --offline --locked -- --exact`. Record your results in [EXPERIMENT.md](EXPERIMENT.md). Each lab runs independently and needs no Docker or external service.

Sources reviewed 2026-10-31. Industry accounts motivate these original adaptations; passing local tests does not certify production readiness.
