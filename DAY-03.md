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
