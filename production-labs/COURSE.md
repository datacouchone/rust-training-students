# Remaining delivery — Days 3, 4 and 5

**Days 1–2 are already delivered and excluded from this rework.** Audience: experienced Rust participants who reported that earlier examples were too straightforward and the pace slow.

Seven 50-minute sessions per day, ten-minute breaks and lunch at 13:00–14:00. These are planned blocks, not validated completion estimates. Begin with a short diagnostic informed by prior Miri coverage; do not repeat the whole introduction.

| Start | Day 3: overload and cancellation | Day 4: performance | Day 5: incidents and release |
|---|---|---|---|
| 10:00 | Industry problem; M6 admission | 8A: baseline and CPU profile | Incident brief and numbered probes |
| 11:00 | Finish M6 main; challenge briefing | 8A: indexed lookup and break-even | Competing hypotheses; partial reads |
| 12:00 | M6 protected-capacity challenge | 8B: collision-safe bounded storage | Framing repair and regression |
| 14:00 | M7 bounded workers | 8B: retention and allocation evidence | Accounting and resource-growth repair |
| 15:00 | Finish M7 main; retry challenge briefing | 8C: scalar and SIMD implementation | Original server logs/task investigation |
| 16:00 | M7 retry challenge; internals as needed | 8C: correctness and measurements | Incident report and rollback criteria |
| 17:00 | Integration and invariant review | Held-out workloads and ship/reject review | Production review |

Begin each day with [the industry problem and module mapping](INDUSTRY-MAP.md), then teach the module concepts needed to solve it. Day 3 now allocates independent attempt time for both challenges; the original executor is supporting/advanced work rather than another compulsory full lab. Full-day completion time has not been rehearsed.

## Current lab inventory

| Day | Modules | New runnable labs | Supporting existing material |
|---|---|---:|---|
| 3 | M6–M7 | 4: two main + M6/M7 challenges | Original atomics/publication and executor labs |
| 4 | M8 | 3: profiling, memory, SIMD | Original SIMD/arena lab |
| 5 | M9–M10 | 1: gateway incident, plus review | Original async-server debugging lab |

Day 4 now has three independent labs. Budget roughly two sessions per lab and one for held-out experiments and review. These are planning estimates; timed classroom rehearsal remains necessary. Faster participants take the explicit advanced-transfer prompts rather than wait.

## Completion evidence

Day 3: explain reservation rollback and task ownership at every cancellation point, including before first poll. Distinguish numeric atomicity from payload publication and capacity limits from fairness.

Day 4: match independent baseline results, record compiler/hardware/input, report measurement variability, and retain or reject the change based on evidence. A SIMD microbenchmark win does not establish an end-to-end service improvement.

Day 5: state trigger, mechanism, blast radius, repair, regression, rollout guard and unknowns. The new probes use deterministic in-memory I/O. Docker remains off; no new Linux core-capture success is claimed.
