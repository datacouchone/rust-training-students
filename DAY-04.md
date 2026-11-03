# Day 4 — three production performance labs

Use the [Day 4 public slides](presentations/day-04/index.html) alongside the lab run sheet.


M8 contains three independent main labs and a final integration challenge. Use the [seven-session course map](production-labs/COURSE.md); old lecture time boxes are supporting notes.

| Lab | Production problem | Deliverable |
|---|---|---|
| [8A: profiling](production-labs/case08a-profile/README.md) | Gateway route lookup grows expensive as policies grow | Profile, indexed lookup, break-even decision |
| [8B: memory](production-labs/case08b-memory/README.md) | Repeated allocations and oversized buffers after a burst | Bounded deduplicated storage, retention measurements |
| [8C: SIMD](production-labs/case08c-simd/README.md) | Nullable column scans with mixed workload sizes | Scalar and SIMD comparison, justified dispatch |
| [Final challenge](day04-challenge/README.md) | A working gateway becomes slow and retains work under load | Repair contracts across Days 1–4; real TCP measurements and regression evidence |

From this repository root:

```sh
cd production-labs
python3 run_case.py case08a-profile --baseline
python3 run_case.py case08b-memory --baseline
python3 run_case.py case08c-simd --baseline
```

Capture each supplied measurement baseline before implementation. Follow each README's correctness and comparison commands. Finish its experiment record with raw results, a held-out workload and a ship/reject decision. Docker is not needed. Existing participant practice is preserved.

## Worked service components

Before each lab, use the [Day 4 production workbench](day04-production/README.md): nine independent demos using ArcSwap, serde_json, Arrow and Criterion. The workbench compares correct implementations and measures workload-dependent tradeoffs. It does not modify the three lab starters. Fetch its locked dependencies before class; run commands are in the workbench README.

## Final integration challenge

The trainer can demonstrate 8A–8C or assign them as participant practice; choose the format in class. The final challenge is independent of their completed code. Reserve approximately 60–90 minutes in pairs, including investigation, with a separate debrief; the duration is a planning estimate. If participants also do all three main labs, adjust their depth or continue unfinished work later.

```sh
python3 day04-challenge/run.py baseline
python3 day04-challenge/run.py healthy
python3 day04-challenge/run.py acceptance
```

The starter compiles and passes a single-request real-TCP smoke test. Acceptance failures expose the intended defects. Read the challenge README for controlled slow/retry/burst/overload workloads and CPU sampling. No Docker is required. The private repair is not in this edition.

## Worked code through slide 16

The instructor has released [the completed examples and Lab 8A reference](DAY-04-THROUGH-SLIDE-16.md). The independent solution leaves your lab attempt in place.

## Four-hour afternoon

Use [the cumulative demos and focused classroom labs](day04-classroom/README.md) for slides 17 onward. Original full labs remain available for follow-up.
