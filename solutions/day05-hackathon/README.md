# Reference solution: Day 5 final hackathon — Project Relay

Published after the classroom exercises at the trainer's request. This directory contains the worked implementation. Your exercise remains at `day05-hackathon/`.

Run from the participant repository root:

```sh
cargo +nightly-2026-07-22 test --manifest-path solutions/day05-hackathon/Cargo.toml --workspace --offline --locked -j 1
```

The pinned toolchain is required, including for portable SIMD. If dependencies have not been downloaded on this machine, run the same command once without `--offline`. Keep the original acceptance tests intact; passing a reduced or edited test is not equivalent to passing the original suite.

Compare files under `src/` with the exercise and explain each repair. The solutions are runnable independently; no copying over your work is required.

## Acceptance and runnable scenarios

The original acceptance suite contains **23 tests**. Run all of them:

```sh
python3 solutions/day05-hackathon/run.py acceptance
python3 solutions/day05-hackathon/run.py baseline
python3 solutions/day05-hackathon/run.py probe slow --clients 12
python3 solutions/day05-hackathon/run.py probe replay --clients 12
python3 solutions/day05-hackathon/run.py probe rollout --clients 12
```

| File | Repair to inspect |
|---|---|
| `src/boundary.rs` | Full identity comparison and input validation. |
| `src/codec.rs` | Preserve partial header/body progress; distinguish clean EOF from truncation; reject oversized frames and poison invalid streams. |
| `src/ledger.rs` | Replay identity, independent bounds, and transactional accounting. |
| `src/worker.rs` | One deadline covers admission and owned work; cancellation drops the operation, active guard and permit; ledger lock begins after the enrichment wait. |
| `src/analytics.rs` | Query correctness and releasing oversized buffer capacity. |
| `src/rollout.rs` | Hold for missing/inconsistent evidence; rollback on duplicate effects; hold for a small sample; then evaluate error and latency thresholds. |

For the inconsistent-evidence test, `successes: 1000, ..healthy()` retains 10 errors and 1000 requests. 1010 outcomes cannot describe 1000 requests, so the decision is Hold. The implementation uses `checked_add` to validate the accounting.

The decoder's original truncation loop checks every incomplete header/body position. Do not replace it with only `n = 1` in a submitted result. Do not manually drop active/permit guards before work finishes just to make counters pass.

These tests establish the fixture's in-process behavior. They do not establish crash durability, remote rollback or exactly-once external effects.
