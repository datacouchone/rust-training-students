# Lab 07 — released worker reference solution

Released by the instructor on 2026-11-02. This is the main M7 worker lab, not challenge 07C. The independent crate contains the completed reference implementation and the original public baseline and acceptance tests. Your attempt in `production-labs/case07-worker` is unchanged.

From the repository root:

```sh
git fetch origin
git restore --source=origin/main -- solutions/case07-worker
cargo +nightly-2026-07-22 test --manifest-path solutions/case07-worker/Cargo.toml --offline --locked
```

Use the workshop's installed pinned nightly and cached dependencies. If dependencies have not been cached, omit `--offline` to let Cargo download the locked versions. No Docker or network service is required for the tests.

## Read the two completed regions

1. `join_deadline` constructs `AbortOnDrop` before returning its future. The future captures the guard, so dropping even an unpolled wrapper requests cancellation. Its timeout path requests abort and awaits the child before returning Deadline. Dropping the wrapper can request abort but cannot await cleanup.
2. `run_batch` validates limits, batch size and unique IDs before spawning; owns at most `limit` tasks in a JoinSet; applies a deadline to each admitted job; and collects terminal outcomes. Explicit cancellation aborts and drains running tasks, then marks accepted IDs without a previously recorded outcome as Cancelled, including queued jobs. A task join error aborts and drains the remaining tasks before returning WorkerPanicked.

Read [the original worker contract](../../production-labs/case07-worker/README.md). Cancellation can report Cancelled for work whose remote effect occurred but whose result was not recorded; it does not undo external effects. Blocking work is outside the abort guarantee. The tests exercise the stated scenarios, not universal production correctness.

Challenge 07C retry solutions are not included in this release.
