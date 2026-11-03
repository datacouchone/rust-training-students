# Project Relay incident handoff

Pair / hardware / toolchain / date:
Driver first half / driver second half:

## Customer-visible failure and competing hypotheses
Record the workload, expected contract and actual observation. Name a plausible diagnosis that evidence ruled out.

## Timeline
| Request/tenant ID | Observation | Hypothesis | Evidence path |
|---|---|---|---|
| | | | |

## Repair log
| Contract | Trigger before repair | Change and ownership argument | Regression after repair | Partner reviewer |
|---|---|---|---|---|
| | | | | |

## Boundary review
Explain pointer validity, readable extent, lifetime, ownership and the actual `memcmp` call. Explain ABI padding versus explicit wire fields. Explain what the validated type guarantees and what it does not guarantee.

## Measurement
Record raw commands, release build, client count, repetitions, success/error counts, p50/p95, active work, retained capacity and a limitation. Compare identical offered workloads. State any workload where the change is worse.

## Release decision
Promote / Hold / Rollback, with evidence:
Minimum observation window and signals:
Stop trigger and responsible person:
How rollback treats already accepted records and receipt state:
How old and new versions coexist:
What remains unsafe to assume about durability or remote effects:

## Five-minute demo
One partner reproduces the failure; the other explains the repair and its limit. Both explain code written by the other. Include two new adversarial regressions.
