# Evidence register — Days 3–5 only

Primary engineering accounts and maintained project documentation support the facts below. Reddit and Stack Overflow were searched for failure reports and misconceptions, then checked against primary documentation and local reproductions. Community claims are not treated as evidence that a company uses a particular implementation. No upstream code was copied into the labs.

| Case | Evidence | What it supports | Our adaptation and limit |
|---|---|---|---|
| 06 | [Amazon load shedding](https://builder.aws.com/content/3Eun1EEyX6p2e3VYNyRLSJzLuMV/using-load-shedding-to-avoid-overload), [Rust atomic ordering](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html) | Early rejection under overload; atomic ordering semantics. | Two weighted capacity counters with ownership-based rollback; not a full fairness/rate-limiting system. |
| 06C | [Amazon load shedding](https://aws.amazon.com/builders-library/using-load-shedding-to-avoid-overload/), [Condvar](https://doc.rust-lang.org/std/sync/struct.Condvar.html) | Overload protection and predicate waiting. | Original two-class capacity policy with live reconfiguration and terminal shutdown; no fairness promise. |
| 07 | [Cloudflare Pingora engineering](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/), [Tokio JoinHandle](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html), [Tokio maintainer discussion 7213](https://github.com/tokio-rs/tokio/discussions/7213) | Upstream efficiency; dropped handles detach tasks; timeout composition needs ownership reasoning. | Bounded synthetic worker with observable cleanup; no proxy/TLS stack. |
| 07C | [Amazon retries/jitter](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/), [Idempotent APIs](https://d1.awsstatic.com/builderslibrary/pdfs/making-retries-safe-with-idempotent-apis-malcolm-featonby.pdf) | Retry amplification, uncertainty and operation identity. | Scripted upstream, injected jitter, shared retry budget and one absolute deadline; no durable deduplication claim. |
| 08A | [Cloudflare Pingora](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/), [flamegraph project](https://github.com/flamegraph-rs/flamegraph) | Proxy efficiency investigations and CPU profiling tools. | Synthetic tenant route lookup with immutable configuration; not Pingora routing internals. |
| 08B | [Discord Read States](https://discord.com/blog/why-discord-is-switching-from-go-to-rust), [Vec clear](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.clear) | Workload-specific allocation investigation; clearing a Vec preserves its capacity. | Collision-safe path storage and explicit retention policy; requested bytes are not RSS. |
| 08C | [Arrow columnar layout](https://arrow.apache.org/docs/format/Columnar.html), [Rust portable SIMD](https://doc.rust-lang.org/core/simd/), [portable SIMD guide](https://github.com/rust-lang/portable-simd/blob/main/beginners-guide.md) | Column buffers, validity bitmaps and portable vector operations. | Original nullable scan format, not Arrow IPC; compare against optimized scalar, no promised speedup. |
| 09 | [Tokio AsyncReadExt](https://docs.rs/tokio/latest/tokio/io/trait.AsyncReadExt.html), [Tokio select cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety) | Individual read operations have different cancellation contracts. | A synthetic fragmented-frame incident with local evidence. No real outage logs or captured production data. |

## Community research trail

- [Stack Overflow: thread/task timeout](https://stackoverflow.com/questions/73450783/is-there-any-thread-spawn-with-timeout) surfaced the difference between timing out work and timing out a handle. The course relies on Tokio documentation and its own cleanup regression tests for the claim.
- [Reddit: hidden control flow in cancellation](https://www.reddit.com/r/rust/comments/1imjjl9/) supplied a discussion prompt about cancellation architecture. Case09 independently reproduces partial-read loss; the thread is not proof of our diagnosis.
- [Reddit: Discord's migration discussion](https://www.reddit.com/r/rust/comments/eytyug/why_discord_is_switching_from_go_to_rust/) motivates asking what a migration result actually establishes. Numerical claims come from neither comments nor our synthetic benchmark.

## Version and evidence discipline

Local code pins Tokio 1.53.1 and its dependencies in Cargo.lock, with nightly-2026-07-22. Public `latest` documentation is a reading reference, not the build definition. The version-specific docs.rs pages for Tokio 1.53.1 were unavailable through the research tool; the installed 1.53.1 source documentation for JoinHandle and AsyncReadExt was checked directly, and the relevant behaviors were reproduced by tests.

GitHub main-branch links and latest documentation can change; reviewed claims and access date are recorded here. Company posts are historical accounts, not assertions about those companies' current deployments. We deliberately omit vendor throughput/security percentages from the core assignments: the exercise is to produce evidence for the supplied implementation.

The presentation sequence and explicit adaptation boundaries are in [INDUSTRY-MAP.md](INDUSTRY-MAP.md). New challenge sources checked 2026-11-01.
