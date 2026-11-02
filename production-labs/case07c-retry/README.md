# M7 challenge — retry without a retry storm

Day 3 · standalone challenge lab · Module 7

## Industry problem

An upstream call fails transiently or commits but loses its reply. Concurrent callers retry, increasing load while deadlines expire. Build a controller that limits retries, retains operation identity and stops cooperatively.

Amazon describes retry amplification, timeout uncertainty, backoff and jitter. Tokio defines cancellation behavior of the futures used here. The scripted upstream and idempotency cache are educational fixtures.

- [Primary source 1](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/)
- [Primary source 2](https://docs.rs/tokio/latest/tokio/macro.select.html)

## Exact contract

Validate before attempting transport: 1–8 attempts, nonzero total timeout, base backoff no larger than maximum backoff, and a nonempty idempotency key if present. One absolute deadline covers every attempt and backoff. The first attempt is free; each additional admitted attempt consumes one token from the shared RetryBudget. Tokens are not refunded after admission. Do not reserve a future retry token while sleeping in backoff.

After completed attempt `n >= 1`, the backoff cap is `min(base × 2^(n-1), max_backoff)` using saturating arithmetic. Clamp the injected jitter result to that cap, then take the larger of the result and retry-after. The absolute deadline still wins when retry-after exceeds remaining time. The test sampler is deterministic; choosing a real randomness source is outside this fixture.

Success stops immediately. Permanent rejection is not retried. RetryableBeforeCommit can retry within all bounds. CommitThenLoseReply sets uncertainty: without an idempotency key stop with Uncertain; with a key every attempt must reuse the original key. Any later terminal error preserves prior uncertainty. Cancellation or deadline during an admitted attempt conservatively reports may_have_committed=true; stopping before the first attempt reports false. This is a result classification, not a claim that a remote effect was rolled back.

The supplied Scripted transport records started/active attempts, committed effects and observed keys. Its tiny cache deduplicates by key for that transport instance only; it is not a durable production idempotency service. Do not alter the transport, RetryBudget, assertions or observations to pass tests.

## Work to complete

1. Compute capped exponential backoff, clamp an injected jitter sample, and honor retry-after as a minimum delay.
2. Keep one absolute deadline across calls and backoff; consume a shared retry token when admitting each extra attempt.
3. Do not retry an uncertain non-idempotent effect; reuse the same key for keyed retries, preserving uncertainty on later failure.
4. Cancel in-flight work and backoff without detaching tasks or spending future retry tokens.

Implement the marked regions in `src/lib.rs`; read adversarial cases in `tests/acceptance.rs`. This crate is independent: no completed main lab is required, but the relevant main lab teaches the prerequisites.

## Run from production-labs/

```sh
cargo +nightly-2026-07-22 test -p case07c-retry --test baseline --offline --locked
python3 run_case.py case07c-retry
# After completing the implementation:
cargo +nightly-2026-07-22 test -p case07c-retry --offline --locked
```

Baseline tests check supplied fixtures only. Starter acceptance failures are intentional. The Python runner limits broken implementations to 90 seconds. Use `--filter TEST_NAME` for one exact acceptance test. No Docker or network service is needed.

## Challenge submission

Write [DECISION.md](DECISION.md), submit a passing regression result, explain one plausible wrong solution and its counterexample, and defend the lifetime/state invariant. The challenge is a standalone crate with its own tests, not merely an extension prompt. Plan a substantial independent attempt; classroom completion time has not been validated.

## Boundaries

Scripted, cooperative in-memory transport; no real upstream request or durable deduplication. Jitter is injected deterministically for testing; a deployment needs suitable independent randomness. No HTTP protocol or general remote cancellation guarantee. The first attempt is admitted without a retry token; attempts counts admission, and cancellation/deadline during an admitted attempt conservatively reports uncertainty.

Sources reviewed 2026-11-01. These are original teaching adaptations; local correctness tests do not certify production readiness.
