# Day 4 — three production performance labs

Use the [Day 4 public slides](presentations/day-04/index.html) alongside the lab run sheet.


M8 contains three independent labs. Use the [seven-session course map](production-labs/COURSE.md); old lecture time boxes are supporting notes.

| Lab | Production problem | Deliverable |
|---|---|---|
| [8A: profiling](production-labs/case08a-profile/README.md) | Gateway route lookup grows expensive as policies grow | Profile, indexed lookup, break-even decision |
| [8B: memory](production-labs/case08b-memory/README.md) | Repeated allocations and oversized buffers after a burst | Bounded deduplicated storage, retention measurements |
| [8C: SIMD](production-labs/case08c-simd/README.md) | Nullable column scans with mixed workload sizes | Scalar and SIMD comparison, justified dispatch |

From this repository root:

```sh
cd production-labs
python3 run_case.py case08a-profile --baseline
python3 run_case.py case08b-memory --baseline
python3 run_case.py case08c-simd --baseline
```

Capture each supplied measurement baseline before implementation. Follow each README's correctness and comparison commands. Finish its experiment record with raw results, a held-out workload and a ship/reject decision. Docker is not needed. Existing participant practice is preserved.
