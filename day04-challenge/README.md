# Final Day 4 challenge — the gateway gets slow under load

A gateway passes its single-request smoke test. Under concurrent traffic, latency climbs. During an upstream slowdown, requests time out but work remains active. A traffic burst leaves more buffer capacity than the service intended to retain.

You are the team investigating the regression. Make the service correct, explain the bottlenecks, then measure the repair. This challenge is independent of completed labs 8A, 8B and 8C.

## What actually runs

The executable starts a load generator, a gateway TCP listener and an upstream TCP listener on **127.0.0.1 with dynamically allocated ports**. These are separate logical components in one process, using a two-worker Tokio runtime and actual TCP connections. There is no Docker, remote service, HTTP, TLS or DNS. The upstream delay is controlled application waiting, not simulated internet packet loss. Requests are read-only lookups, so the fixture has no remote write effects to deduplicate.

Request path: client → framed JSON → admission → routing-policy access → route lookup → upstream attempt/retry → telemetry → response. Each client response records queue, routing and upstream-path durations. Client latency includes the complete socket exchange. The upstream-path duration includes retries and backoff; it does not isolate pure network transport time. Stage percentiles cannot be added to obtain total p95.

## Start here

From the student repository root:

```sh
python3 day04-challenge/run.py baseline
python3 day04-challenge/run.py healthy
python3 day04-challenge/run.py slow
python3 day04-challenge/run.py retry
python3 day04-challenge/run.py burst
python3 day04-challenge/run.py overload
python3 day04-challenge/run.py acceptance
```

Dependencies must be cached. First setup, if needed: `cargo +nightly-2026-07-22 fetch --manifest-path day04-challenge/Cargo.toml --locked`. The runner builds with `--offline --locked` and applies a watchdog. Baseline must pass. The initial acceptance failures are deliberate; **do not delete tests, change fixtures, or turn failures into ignored tests**.

For Activity Monitor, run `python3 day04-challenge/run.py profile`: it prints its PID and repeats the workload for roughly 20 seconds. Search for `day04-challenge` under CPU. On macOS, `python3 day04-challenge/run.py sample` captures that process into `day04-challenge/results/cpu-sample.txt`. No CPU core is pinned. Run one measurement at a time.

## The contract you must restore

1. Admission tracks logical payload bytes. Rejection changes neither used bytes nor accepted count. An owned lease releases its bytes on return, timeout and cancellation. Overflow must reject before mutation.
2. The shared retry budget admits at most its initial number of extra attempts, remains zero when exhausted, and behaves correctly under contention. Initial attempts do not consume retry tokens. The supplied deterministic stagger is a repeatable fixture, not production random jitter.
3. One deadline starts at gateway handling and covers policy-lock waiting, upstream attempts and retry backoff. Input frame reading has a separate supplied fixture timeout. Returning timeout or cancelling the parent must leave no locally owned attempt running. Remote cancellation observation can occur later; local cancellation is not remote rollback.
4. Independent requests must reach the gated upstream together. Routing-policy access must not serialize upstream waits.
5. Route by longest complete segment prefix; tenant specificity breaks ties at the same prefix. Tenant 0 is wildcard. Preserve root/pool zero and misses. Lookup work should depend on path depth, not unrelated policy size. Preserve the `inspected` counter as an honest count of candidate prefixes examined.
6. Flush empties the telemetry buffer. Capacity above `retain` is discarded; smaller allocations remain reusable. This bounds the retained byte buffer, not RSS or all service memory.
7. The telemetry query selects only valid rows with latency >= threshold and status in 500..=599. Count and total refer to the same rows; widen values before summation. Inputs are bounded by the supplied client fixture. Removing intermediate collections is worth measuring; SIMD is optional, not a grading requirement.

## Investigation, then implementation

Read `INVESTIGATION.md` and record the baseline **before** editing. The editable service files are `src/policy.rs`, `src/gateway.rs`, and `src/telemetry.rs`. They contain working but faulty code, not empty TODOs. Keep the transport fixture, main runner, metrics and tests intact. Add regression tests when an additional edge case matters.

Use one exact regression while repairing, for example:

```sh
cargo +nightly-2026-07-22 test --manifest-path day04-challenge/Cargo.toml --offline --locked --test acceptance rejected_admission_does_not_change_state -- --exact
```

`HINTS.md` provides progressively stronger hints. Start with the symptom and evidence, not a blanket rewrite. Record one plausible diagnosis you ruled out.

## What to submit

Your patch; all acceptance results; before/after raw workload output with hardware, toolchain and concurrency; p50/p95 with outcome counts; resource/attempt observations; a sampled stack or a stated collection limitation; and a ship/reject decision. Compare the same offered workload and successful outcomes: rejecting everything is not a performance improvement.

The harness is a small **closed-loop batch**, not an open-loop production load generator. Its throughput counts all completed responses, including errors; report successes separately from outcomes. Small-sample p95 is an observation, not an SLO. Do not equate requested Vec capacity with RSS. Increasing deadlines, admission limits, worker count or retention thresholds to hide the symptom does not satisfy the contract.

Plan 60–90 minutes in pairs, then review; this is an estimate, not a timed classroom rehearsal. Keep 8A–8C available for individual practice before or after this challenge. No completed earlier lab is required.

## How the four days meet

| Work | Earlier concepts |
|---|---|
| Lease lifetime, borrowed policy lookup, buffer ownership | Day 1 ownership, layout and Drop |
| Validation, explicit error outcomes, state-transition contracts | Day 2 invariant-driven API design |
| Contended budget, lock scope, futures, deadlines, cancellation | Day 3 atomics and async ownership |
| Stage timing, CPU sampling, indexing, retention, column predicates | Day 4 performance engineering |

Industry motivation: [Cloudflare Pingora](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/), [Amazon retries and jitter](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/), and [Discord Read States](https://discord.com/blog/why-discord-is-switching-from-go-to-rust). This is an original synthetic incident, not their implementation or a reproduction of a company outage. Tokio's pinned source defines the runtime contracts. Day 5 retains its separate framing/cancellation incident and deployment review.
