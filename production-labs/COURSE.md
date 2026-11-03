# Remaining delivery — Days 3, 4 and 5

**Days 1–2 are already delivered and excluded from this rework.** Audience: experienced Rust participants who reported that earlier examples were too straightforward and the pace slow.

Days 3–4 use seven 50-minute sessions and ten-minute breaks. Day 5 reserves a continuous 150-minute paired exercise after lunch. Lunch is 13:00–14:00. These are planned blocks, not validated completion estimates. Begin with a short diagnostic informed by prior Miri coverage; do not repeat the whole introduction.

| Start | Day 3: overload and cancellation | Day 4: performance | Day 5: incidents and release |
|---|---|---|---|
| 10:00 | Industry problem; M6 admission | 8A: baseline and CPU profile | Incident brief and numbered probes |
| 11:00 | Finish M6 main; challenge briefing | 8A: indexed lookup and break-even | Competing hypotheses; partial reads |
| 12:00 | M6 protected-capacity challenge | 8B: collision-safe bounded storage | Framing repair and regression |
| 14:00 | M7 bounded workers | 8B: retention and allocation evidence | Project Relay starts: baseline and first repair |
| 15:00 | Finish M7 main; retry challenge briefing | 8C: scalar and SIMD implementation | Project Relay: swap roles and integrate repairs |
| 16:00 | M7 retry challenge; internals as needed | 8C: measurements; optional challenge start | Project Relay: evidence and release decision |
| 17:00 | Integration and invariant review | Final gateway challenge and evidence review | Pair demos and production review |

Begin each day with [the industry problem and module mapping](INDUSTRY-MAP.md), then teach the module concepts needed to solve it. Day 3 now allocates independent attempt time for both challenges; the original executor is supporting/advanced work rather than another compulsory full lab. Full-day completion time has not been rehearsed.

## Current lab inventory

| Day | Modules | New runnable labs | Supporting existing material |
|---|---|---:|---|
| 3 | M6–M7 | 4: two main + M6/M7 challenges | Original atomics/publication and executor labs |
| 4 | M8 + Days 1–4 integration | 4: profiling, memory, SIMD, gateway challenge | Original SIMD/arena lab |
| 5 | M9–M10 + Days 1–5 integration | 2: gateway incident + paired Project Relay | Original async-server debugging lab |

Day 4 has three independent main labs plus the final gateway challenge in `../day04-challenge/`. The main labs may be trainer demonstrations or participant practice; the final challenge does not require completed earlier solutions. Reserve about 60–90 minutes for the challenge, adjusting the earlier lab depth to fit. Full completion of all four exercises is not promised within the existing timetable. These are planning estimates; timed classroom rehearsal remains necessary.

## Completion evidence

Day 3: explain reservation rollback and task ownership at every cancellation point, including before first poll. Distinguish numeric atomicity from payload publication and capacity limits from fairness.

Day 4: match independent baseline results, record compiler/hardware/input, report measurement variability, and retain or reject the change based on evidence. A SIMD microbenchmark win does not establish an end-to-end service improvement.

Day 5: state trigger, mechanism, blast radius, repair, regression, rollout guard and unknowns. Lab 09 uses deterministic in-memory I/O; Project Relay adds native loopback TCP and deterministic cancellation regressions. Docker remains off; no new Linux core-capture success is claimed.

## Day 5 paired finale

Project Relay lives in `../day05-hackathon/`. Reserve **150 uninterrupted exercise minutes**, with a role swap at minute 50, evidence work at minute 85 and release review at minute 115. The suggested afternoon window is 14:00–16:30; this replaces three isolated 50-minute blocks for Day 5. Add group-demo time afterwards. Two-person teams integrate Days 1–5; no earlier completed solution is required. Docker/core rehearsal remains deferred.
