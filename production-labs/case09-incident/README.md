# An ingestion incident that survives a happy-path test

Delivery Day 5 · Production case 09

A gateway reports intermittent decode errors after deadlines, inconsistent request totals after rejected records, and memory growth from new tenants.

## Evidence and adaptation

Tokio documents cancellation safety for individual read operations. Community reports highlight how select/timeout complicates control flow. Our incident fixture produces its own synthetic observations.

- [Source 1](https://docs.rs/tokio/latest/tokio/io/trait.AsyncReadExt.html)
- [Source 2](https://www.reddit.com/r/rust/comments/1imjjl9/)

This is an original classroom implementation. Company sources establish the motivating problem or technique; they do not endorse this code or establish production readiness. Sources checked 2026-10-31.

## Your contract

1. Run the three numbered probes and record competing hypotheses before editing. The participant source starts functional but faulty.
2. Preserve consumed header/body bytes across cancelled reads. Reject oversized/truncated frames and poison a terminally invalid stream.
3. Repair atomic accounting updates and cardinality enforcement. Reproduce the original trigger and add regression evidence.

Start with `src/lib.rs`; acceptance scenarios are in `tests/acceptance.rs`.
Implementation sequence: **investigate -> next -> apply**.

## Run from this production-labs directory

```sh
cargo +nightly-2026-07-22 test -p case09-incident --test baseline --offline --locked
cargo +nightly-2026-07-22 test -p case09-incident --test acceptance --offline --locked
```

The baseline checks supplied infrastructure only. Acceptance failures are expected initially. Work on one named test at a time; passing baseline or compilation does not complete the case.

Start investigation before editing:

```sh
cargo +nightly-2026-07-22 run -p case09-incident --bin probe --offline --locked -- 1
cargo +nightly-2026-07-22 run -p case09-incident --bin probe --offline --locked -- 2
cargo +nightly-2026-07-22 run -p case09-incident --bin probe --offline --locked -- 3
```

Record observations in `INVESTIGATION.md`. Probe commands intentionally exit successfully when they print a faulty observation; use acceptance tests to establish a repair. These probes need no Docker or external service.

## Adversarial acceptance

- Cancellation after every byte, adjacent frames, empty body and clean EOF.
- Oversized lengths rejected before body allocation, poisoned retries and all truncations.
- Overflow updates and tenant-cardinality growth.

## Advanced transfer

Rerun the original Lab 9 server incidents with structured logs and task inspection. Compare native thread stacks with logical async state using the supplied core-dump guide only when Docker rehearsal is authorized. Prepare a bounded retry/rollback incident variant.

This transfer is an open design/implementation task, not an already supplied acceptance-tested extension. Specify new acceptance evidence before coding.

## Finish with evidence

Submit your change, a previously failing case, a passing regression check, the invariant or ownership argument, and one remaining limitation. Explain a plausible fix you rejected and why. Keep the API and tests; do not delete assertions or relax the contract to pass.

## Boundaries

Uses Tokio in-memory duplex I/O to make packet fragmentation deterministic. It is not a network load test. Live Docker/core capture is deferred; do not start Docker for this track.
