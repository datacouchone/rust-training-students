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
