# Day 3 — admission, task ownership and async internals

Use the [Day 3 public slides](presentations/day-03/index.html) alongside the lab run sheet.


Use the [production course map](production-labs/COURSE.md) for today's seven 50-minute sessions. Start with [case06-admission](production-labs/case06-admission/README.md). The old module lecture time boxes are superseded by this schedule.

From the participant repository root:

```sh
cd production-labs
python3 run_case.py case06-admission --baseline
```

Read the case contract, predict the first failure, then attempt it. Core acceptance and advanced transfer work are separate. Submit a regression result and invariant argument, not just completed code. Do not reset earlier practice files.

Then continue with [bounded workers](production-labs/case07-worker/README.md). Prior Miri material makes the opening diagnostic short; it does not replace an ordering or cancellation argument.


## Two standalone challenges

After the M6 main lab: [06C — protected critical capacity](production-labs/case06c-reserve/README.md). After the M7 main lab: [07C — bounded, idempotency-aware retries](production-labs/case07c-retry/README.md). Each has a separate crate, starter, acceptance suite and decision worksheet. This gives Day 3 **two main labs and two challenge labs**.

Start with [industry problem → modules → labs](production-labs/INDUSTRY-MAP.md#day-3--survive-overload-and-uncertain-upstream-results). The seven-block plan reserves time for challenge attempts; optional executor and open transfer work must fit the actual cohort pace.

## Short teaching demonstrations

Slides 8–15 use eight separate programs, so each concept has its own before/after demonstration. These are small executable models of specific failure mechanisms, not complete production gateways or extra assignments. The four lab assignments above are unchanged.

From the repository root, run a model with:

```sh
cargo +stable run --manifest-path concepts/Cargo.toml --bin day3_arc
```

| Slide | Program | What it isolates |
| --- | --- | --- |
| 8 | `day3_atomic_decision` | Why SeqCst does not combine a check and update |
| 9 | `day3_cas` | Failed compare-and-swap and rechecking capacity |
| 10 | `day3_drop` | Returning capacity when the owner leaves scope |
| 11 | `day3_rollback` | Returning global capacity after tenant refusal |
| 12 | `day3_held` | Testing reservations while threads still hold them |
| 13 | `day3_publication` | Separating capacity accounting from payload transfer |
| 14 | `day3_arc` | Sharing one budget, Send/Sync bounds and owner lifetime |
| 15 | `day3_policy` | Protecting a combined policy check and update with one lock |

The separate poll, wake, Pin and RawWaker models support M7. A successful process exit in a deliberately faulty default model does not mean the policy is correct: compare the printed state with the invariant.

## M7: a separate replacement on slides 19–26

Each slide below has its own Tokio example. Run its default to expose the missing guarantee, then compare the changed version shown by the trainer. These are focused experiments; Lab 07 remains a separate assignment. Each run includes assertions about the expected observation. A deliberately faulty default can exit successfully because its assertion confirms the faulty state.

```sh
cargo +nightly-2026-07-22 run --manifest-path production-labs/Cargo.toml -p case07-worker --example day3_handle_detach --offline --locked
```

| Slide | Example | Default → changed observation |
| --- | --- | --- |
| 19 | `day3_waiting_resources` | Waiting request incorrectly counted as 0 → its guard keeps it counted as 1 until completion |
| 20 | `day3_future_lifetime` | Guard inside an unpolled body never exists → captured guard exists at construction and drops with the future |
| 21 | `day3_handle_detach` | Dropped handle leaves the child active → abort plus join observes cleanup |
| 22 | `day3_unpolled_guard` | Guard inside an unpolled wrapper never exists → construct it before returning the wrapper |
| 23 | `day3_abort_observed` | Read immediately after abort → read after awaited termination |
| 24 | `day3_bounded_spawn` | Spawned/unjoined population reaches 4 → waiting before spawn keeps it at 2 |
| 25 | `day3_cancel_ledger` | Queued accepted job disappears → reconcile every accepted ID while preserving recorded success |
| 26 | `day3_deadline_cleanup` | Timeout assertion passes with a child still active → timeout path aborts, joins and checks resource release |

The harness uses a current-thread Tokio runtime, handshakes and a paused virtual clock. It is not a throughput benchmark. The immediate-after-abort observation specifically depends on that controlled runtime; a multithreaded runtime may already have processed cancellation. Faulty examples explicitly clean up after displaying their observation.

The four poll/wake/Pin/RawWaker examples remain on slides 28–31. Slide 27 reviews the full worker lab; slides 33–34 review the retry challenge. Those reference reviews still replace the named lab bodies.

Primary API references: [Tokio JoinHandle](https://docs.rs/tokio/1.53.1/tokio/task/struct.JoinHandle.html), [JoinSet](https://docs.rs/tokio/1.53.1/tokio/task/struct.JoinSet.html), and [paused-time behavior](https://docs.rs/tokio/1.53.1/tokio/time/fn.advance.html).

## Released challenge reference

[06C reference solution and run commands](solutions/case06c-reserve/README.md) are now available in a separate crate. Keep your own challenge attempt in `production-labs/case06c-reserve`.
