# Day 5 — incident diagnosis and production review

Use the [Day 5 public slides](presentations/day-05/index.html) alongside the lab run sheet.


Use the [production course map](production-labs/COURSE.md) for today's seven 50-minute sessions. Start with [case09-incident](production-labs/case09-incident/README.md). The old module lecture time boxes are superseded by this schedule.

From the participant repository root:

```sh
cd production-labs
python3 run_case.py case09-incident --baseline
```

Read the case contract, predict the first failure, then attempt it. Core acceptance and advanced transfer work are separate. Submit a regression result and invariant argument, not just completed code. Do not reset earlier practice files.

Run the numbered probes before editing and fill in the [incident record](production-labs/case09-incident/INVESTIGATION.md). Finish with [production review](production-labs/REVIEW.md). Docker/core capture is deferred; the probes require no Docker.
