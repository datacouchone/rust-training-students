# Industry problems → modules → labs

The remaining three days follow one gateway/service lifecycle: survive overload, measure and optimize real work, then investigate failures and release a repair. Published accounts motivate the problems. The lab code, request traces and incidents are original synthetic teaching models, not copies of vendor internals or real outage data.

## Day 3 — survive overload and uncertain upstream results

**Problem:** uploads and upstream calls consume finite capacity. A tenant can crowd out others; critical traffic still needs room. When an upstream slows down, timed-out work and simultaneous retries can multiply the load. A lost response may follow a committed effect.

**Modules:** M6 makes shared-resource decisions correct under contention, live configuration changes and shutdown. M7 owns async work across deadlines, cancellation and retries. Memory safety alone does not establish those application guarantees.

| Lab | Module | Production use case | Published motivation | Evidence participants produce |
|---|---|---|---|---|
| 06 — main | M6 | Weighted global/tenant upload admission | Amazon load shedding | Held reservations never overbook; tenant rejection and unwind return capacity |
| 06C — challenge | M6 | Protect critical traffic during overload and live limit changes | Amazon overload protection; Rust Mutex/Condvar contracts | Reserved capacity, atomic reconfiguration, bounded waiting and shutdown wakeups |
| 07 — main | M7 | Bounded upstream fan-out with deadlines | Cloudflare Pingora efficiency context; Tokio task lifetime | Every input has an outcome; timed-out and cancelled work is owned and cleaned up |
| 07C — challenge | M7 | Retry transient upstream failures without amplification or duplicate effects | Amazon timeouts/backoff/jitter and idempotent APIs | Shared retry budget, absolute deadline, stable key and preserved uncertainty |

Sources: [Amazon load shedding](https://aws.amazon.com/builders-library/using-load-shedding-to-avoid-overload/), [Cloudflare Pingora](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/), [Tokio JoinHandle](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html), [Amazon retries and jitter](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/), [Amazon idempotent APIs](https://d1.awsstatic.com/builderslibrary/pdfs/making-retries-safe-with-idempotent-apis-malcolm-featonby.pdf).

**What is adapted:** the two-class reserve policy and scripted retry transport are our design. Cloudflare is not cited as using this worker algorithm. A tiny in-memory key cache is not a production durable deduplication service. Capacity protection is not fairness; cancellation is not remote rollback.

## Day 4 — decide which optimization belongs in the service

**Problem:** growing routing policies increase CPU work; repeated telemetry strings cause allocation churn; a burst leaves oversized buffers; nullable scans make naive SIMD produce incorrect results. A faster microbenchmark can still lose on another workload or worsen retention.

**Module:** M8 uses profiling to choose a target, memory measurements to understand allocation/retention, and scalar/vector comparisons to make a deployment decision while preserving semantics.

| Lab | Module | Production use case | Published motivation | Evidence participants produce |
|---|---|---|---|---|
| 8A — profiling | M8 | Gateway route lookup under growing policy size | Cloudflare proxy-efficiency account | CPU profile, exact route equivalence, index build cost and break-even |
| 8B — memory | M8 | Telemetry batching with repeated paths and burst retention | Discord Read States investigation; Vec capacity contract | Collision correctness, transactional quotas, peak/retained requested bytes |
| 8C — SIMD | M8 | Nullable columnar query filtering | Apache Arrow columnar/validity format; Rust portable SIMD | Scalar equivalence, mask/tail/overflow tests, measured dispatch decision |

Sources: [Cloudflare Pingora](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/), [Discord Read States](https://discord.com/blog/why-discord-is-switching-from-go-to-rust), [Arrow columnar format](https://arrow.apache.org/docs/format/Columnar.html), [Rust portable SIMD](https://doc.rust-lang.org/core/simd/).

**What is adapted:** synthetic route policy and telemetry formats, not Pingora routing internals, Discord's cache or Arrow IPC. Local timing is not vendor performance or service p99. A measured rejection of an optimization is a valid outcome.

## Day 5 — explain an incident and release a defensible repair

**Problem:** a deadline interrupts a frame read and a retry starts at the wrong byte; rejected records still change accounting; new tenants grow retained state. A happy-path test misses these transitions. Shipping a repair requires evidence beyond an error disappearing once.

**Modules:** M9 localizes the mechanism using deterministic probes, structured context, task ownership and native/async evidence. M10 reviews regressions, uncertain effects, staged rollout and rollback criteria.

| Lab or review | Module | Production use case | Published motivation | Evidence participants produce |
|---|---|---|---|---|
| 09 — incident | M9 | Telemetry gateway framing, accounting and resource growth | Tokio read cancellation contracts; tracing/task tooling | Competing hypotheses, every-byte cancellation regression, transactional update and cardinality bounds |
| Existing server drill | M9 supporting | Deadlock, leaked tasks and panic investigation | Tokio/tracing documentation and the supplied original server | Lock graph, task-lifetime explanation, matching operational evidence |
| Release review | M10 | Roll out a repair without turning a local fix into a wider incident | Amazon safe deployment and retry guidance | Customer-impact statement, canary metric, rollback trigger and missing evidence |

Sources: [Tokio read cancellation](https://docs.rs/tokio/latest/tokio/io/trait.AsyncReadExt.html), [tracing instrument](https://docs.rs/tracing/latest/tracing/attr.instrument.html), [tokio-console](https://github.com/tokio-rs/console), [Amazon safe deployments](https://d1.awsstatic.com/builderslibrary/pdfs/automating-safe-hands-off-deployments-clareliguori.pdf).

**What is adapted:** the gateway incident is synthetic; it is not presented as a Cloudflare, Discord or Amazon outage. The release review applies published principles; it does not deploy a service. Docker/core capture remains deferred.

## Evidence boundaries

Sources checked 2026-11-01. Historical engineering articles establish the motivating issue or technique, not a claim about current deployments. Local code uses pinned nightly-2026-07-22 and Tokio 1.53.1. The public latest documentation is a reading reference; tests and pinned source determine the exercise behavior.
