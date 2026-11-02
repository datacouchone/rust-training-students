# M6 challenge — protected capacity during overload

Day 3 · standalone challenge lab · Module 6

## Industry problem

A shared gateway must leave room for critical control traffic while ordinary uploads surge. Operators change capacity while requests are alive, and shutdown must wake blocked callers without revoking existing owners.

Amazon documents early load shedding and capacity protection during overload. Our two-class reservation policy and hot-reconfiguration exercise are original adaptations, not an AWS implementation.

- [Primary source 1](https://aws.amazon.com/builders-library/using-load-shedding-to-avoid-overload/)
- [Primary source 2](https://doc.rust-lang.org/std/sync/struct.Condvar.html)

## Exact contract

`Config { capacity, critical_reserve }` must have positive capacity and reserve no larger than capacity. A Normal admission requires `normal + critical + weight <= capacity` and `normal + weight <= capacity - critical_reserve`. Critical admission uses only the total bound. Zero weight is rejected; a weight larger than its class ceiling is TooLarge, while a temporarily unavailable fitting weight is Busy.

Rejection leaves the whole snapshot unchanged. Reconfigure validates the entire candidate against both current counters before replacing configuration. A smaller capacity or larger reserve that violates live usage returns WouldOvercommit. Existing leases are never revoked or repriced. Close is idempotent and terminal: it prevents future acquisition and reconfiguration, wakes waiters with Closed, and lets existing leases release normally.

Wait under a Condvar predicate loop with a single absolute deadline. Zero wait may succeed immediately if capacity is available; otherwise it returns Timeout. Wakeups from release, reconfiguration, close or spuriously from the OS require a fresh predicate check. Release must notify all because waiter classes and weights differ. This is deliberately a Mutex/Condvar challenge: the state transition covers a compound invariant, unlike the main lab's individual atomic counters.

## Work to complete

1. Implement admission under one lock: total usage stays within capacity and normal usage stays within capacity minus critical reserve.
2. Implement bounded waiting using one absolute deadline and a rechecked predicate after every notification.
3. Make live reconfiguration transactional, closure terminal and release ownership correct through unwind.

Implement the marked regions in `src/lib.rs`; read adversarial cases in `tests/acceptance.rs`. This crate is independent: no completed main lab is required, but the relevant main lab teaches the prerequisites.

## Run from production-labs/

```sh
cargo +nightly-2026-07-22 test -p case06c-reserve --test baseline --offline --locked
python3 run_case.py case06c-reserve
# After completing the implementation:
cargo +nightly-2026-07-22 test -p case06c-reserve --offline --locked
```

Baseline tests check supplied fixtures only. Starter acceptance failures are intentional. The Python runner limits broken implementations to 90 seconds. Use `--filter TEST_NAME` for one exact acceptance test. No Docker or network service is needed.

## Challenge submission

Write [DECISION.md](DECISION.md), submit a passing regression result, explain one plausible wrong solution and its counterexample, and defend the lifetime/state invariant. The challenge is a standalone crate with its own tests, not merely an extension prompt. Plan a substantial independent attempt; classroom completion time has not been validated.

## Boundaries

OS-thread blocking API; do not invoke wait_acquire directly on an async runtime worker. Quotas are resource units, not measured process RSS. No fairness, preemption or revocation of existing leases is promised. Timing tests use real OS threads; the watchdog bounds hangs.

Sources reviewed 2026-11-01. These are original teaching adaptations; local correctness tests do not certify production readiness.
