# Bounded fan-out and cancellation ownership

Delivery Day 3 · Production case 07

An upstream dependency slows down. Requests time out, but spawned tasks continue consuming resources. A batch cancellation must account for queued and running work.

## Evidence and adaptation

Tokio documents JoinHandle detachment and task abortion; its maintainer discussion explains the timeout-on-handle trap. Pingora motivates upstream efficiency; this worker is an original model, not its implementation.

- [Source 1](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html)
- [Source 2](https://github.com/tokio-rs/tokio/discussions/7213)
- [Source 3](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/)

This is an original classroom implementation. Company sources establish the motivating problem or technique; they do not endorse this code or establish production readiness. Sources checked 2026-10-31.

## Your contract

1. Validate batch size and unique IDs before spawning. Bound the number of owned running tasks.
2. Apply deadlines to work, return every request outcome, and release all running work before reporting explicit cancellation.
3. Own a spawned task across a deadline and parent-future drop. Aborting must be followed by joining where the API promises cleanup has completed.

Start with `src/lib.rs`; acceptance scenarios are in `tests/acceptance.rs`.
Implementation sequence: **join_deadline -> run**.

## Run from this production-labs directory

```sh
cargo +nightly-2026-07-22 test -p case07-worker --test baseline --offline --locked
cargo +nightly-2026-07-22 test -p case07-worker --test acceptance --offline --locked
```

The baseline checks supplied infrastructure only. Acceptance failures are expected initially. Work on one named test at a time; passing baseline or compilation does not complete the case.

## Adversarial acceptance

- Virtual-time deadline tests, queue cancellation and already-cancelled batches.
- Failure outcomes mixed with successful jobs.
- Timed-out JoinHandle and dropped parent; active/peak counters expose resource ownership.

## Advanced transfer

Combine case06 byte budgets with the worker. Then add a retry budget and classify idempotent versus unsafe retries against the documented idempotency contract. Full raw-waker implementation remains the existing Lab 7 advanced internals drill.

This transfer is an open design/implementation task, not an already supplied acceptance-tested extension. Specify new acceptance evidence before coding.

## Finish with evidence

Submit your change, a previously failing case, a passing regression check, the invariant or ownership argument, and one remaining limitation. Explain a plausible fix you rejected and why. Keep the API and tests; do not delete assertions or relax the contract to pass.

## Boundaries

Deadline starts when a job is admitted, not at batch submission. Input and result memory are bounded by MAX_BATCH, not streamed. Cancellation can report Cancelled for a side effect whose result was not observed; it does not roll back remote work. Blocking work is outside the abort guarantee.
