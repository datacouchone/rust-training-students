# Production Rust labs — Days 3–5 only

Days 1–2 have already been delivered. This active rework covers Modules 6–10 only; original delivered labs and participant practice remain unchanged.

| Day | Module | Active new lab | Implementation tasks |
|---|---|---|---:|
| 3 | M6 concurrency/atomics | [06: Multi-tenant admission](case06-admission/README.md) | 3 |
| 3 | M6 challenge | [06C: Protected critical capacity](case06c-reserve/README.md) | 5 |
| 3 | M7 async | [07: Bounded worker and cancellation](case07-worker/README.md) | 2 |
| 3 | M7 challenge | [07C: Retry without amplification](case07c-retry/README.md) | 2 |
| 4 | M8 performance | [8A — Profile an edge gateway route lookup](case08a-profile/README.md) | 2 |
| 4 | M8 performance | [8B — Bound telemetry memory after a traffic burst](case08b-memory/README.md) | 3 |
| 4 | M8 performance | [8C — Decide whether SIMD belongs in a columnar scan](case08c-simd/README.md) | 3 |
| 5 | M9 debugging | [09: Gateway incident](case09-incident/README.md) | 2 repair areas |
| 5 | M10 production practice | [Release review](REVIEW.md) | Review exercise |

**Current total: six main coding labs, two standalone challenge labs and one review exercise.** Day 3 has two main labs plus one M6 and one M7 challenge. Day 4 has three separate runnable crates, each with its own contract, starter, acceptance suite, measurement binary and experiment record. The original SIMD/arena lab remains supporting material.

Start with [industry problems → modules → labs](INDUSTRY-MAP.md). See [the remaining-day course map](COURSE.md) and [source register](SOURCES.md). Industry sources motivate original teaching adaptations; the code is not a company's production implementation.

## Setup and commands

From this directory, use pinned nightly-2026-07-22 and Python 3. No Docker, credentials or external service is needed. Fetch once on a new machine:

```sh
cargo +nightly-2026-07-22 fetch --locked
cargo +nightly-2026-07-22 check --workspace --all-targets --offline --locked
cargo +nightly-2026-07-22 test --workspace --test baseline --offline --locked
python3 run_case.py case06-admission
```

Baseline tests check supplied infrastructure only. Acceptance failures are expected initially. The runner has a 90-second process-group watchdog and rejects unknown exact filters. Work one failing case at a time; keep the contract and tests intact. Submit a regression result, an invariant argument and remaining limitations.

The original labs under ../labs remain supporting internals drills. Advanced transfer prompts in the new cases are open assignments, not extra prebuilt labs. Docker/core capture remains deferred.
