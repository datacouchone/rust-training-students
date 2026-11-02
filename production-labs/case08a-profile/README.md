# 8A — Profile an edge gateway route lookup

Day 4 · M8 · independent production-use-case lab

The gateway routes each tenant request to an upstream pool. A larger route policy makes a formerly cheap linear lookup dominate request processing. Diagnose the supplied pipeline, replace lookup without changing policy semantics, and explain whether snapshot construction pays for itself.

Cloudflare's Pingora account motivates investigating proxy efficiency; this route-policy model is original classroom code, not a reconstruction of Pingora.

## Contract and difficult cases

Implement `Index::compile` and `Index::route`. Match the longest **segment** prefix first; a tenant-specific rule wins only a tie at that prefix. `/api` cannot match `/apiary`. A shorter tenant rule cannot beat a longer wildcard rule. Root and pool zero are valid. Duplicate tenant/prefix pairs are errors. Prefixes must be absolute, contain no query, fragment or doubled slash, and have no trailing slash except root. Invalid request paths return `None`. Unicode paths work; no decoding or normalization is supplied. The compiled snapshot owns its configuration.

Before editing, use stage timings to name the likely bottleneck, then collect a sampling profile. Account separately for parsing, lookup, aggregation and index construction. Preserve the independent linear baseline. Validate all outputs before timing. No per-request allocation is needed in the indexed lookup.

## Investigation sequence

1. Capture seven baseline samples and a CPU profile; predict the dominant stack.
2. Run `segment_boundary_and_longest_prefix` and `tenant_ties_do_not_override_longer_prefix`, then implement the snapshot and lookup.
3. Run all acceptance tests. Compare seven alternating-order measurements with 256 routes and again with 4 routes.
4. Estimate break-even request count from index construction cost divided by per-request savings. If savings are nonpositive, report that no break-even was established.
5. Use a held-out route count and path depth. Decide whether to ship, restrict use, or retain the linear path.

The runner prints stage durations as instrumentation. These are not a sampling profile or service latency percentiles. The index is built outside steady-state timing and reported on stderr. Snapshot replacement, concurrent reload and real HTTP parsing are out of scope.

## CPU sampling

On macOS, run `target/release/profile-route profile-baseline 10000 256` in one terminal, then `sample PID 1 1 -file route-baseline.txt` in another, using the PID of that process. Repeat with `profile-candidate` after implementation. Each process loops for three seconds. If attachment is unavailable, record that limitation; do not label stage timing a flamegraph. On supported Linux setups, the optional [cargo-flamegraph project](https://github.com/flamegraph-rs/flamegraph) documents installation and permissions; no installation or privileged setting is required by this lab.

## Advanced transfer

Design immutable policy replacement under concurrent readers. Specify reload atomicity, old-snapshot lifetime, peak memory during rebuild and rejection of invalid configuration. Add tests before implementation; this extension is not supplied.

## Commands

From `production-labs/`, before editing:

```sh
cargo +nightly-2026-07-22 test -p case08a-profile --test baseline --offline --locked
cargo +nightly-2026-07-22 run --release -p case08a-profile --bin profile-route --offline --locked -- baseline 10000 256 > baseline.csv
```

After implementation:

```sh
cargo +nightly-2026-07-22 test -p case08a-profile --offline --locked
cargo +nightly-2026-07-22 run --release -p case08a-profile --bin profile-route --offline --locked -- compare 10000 256 > compare.csv
```

Acceptance failures are expected in the starter. Keep APIs and assertions intact. Work one named test at a time with `cargo +nightly-2026-07-22 test -p case08a-profile --test acceptance TEST_NAME --offline --locked -- --exact`. Record your results in [EXPERIMENT.md](EXPERIMENT.md). Each lab runs independently and needs no Docker or external service.

Sources reviewed 2026-10-31. Industry accounts motivate these original adaptations; passing local tests does not certify production readiness.
