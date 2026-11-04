# Investigation record

Record your machine, Rust toolchain, source revision, concurrency and command. Save raw output under `results/`. The smallest workload is a smoke test, not a performance result.

| Workload | Symptom / measured outcome | Two possible explanations | Experiment that separates them | Repair evidence |
|---|---|---|---|---|
| healthy | | | | |
| slow | | | | |
| retry | | | | |
| burst | | | | |
| overload | | | | |

Before editing, answer: Is the time in queueing, routing, or the upstream path? What remains alive after a timeout? What changed despite a rejection? Which capacities remain after a flush? Does the summary describe the same rows as the scalar contract?

After editing, compare identical workloads. Report outcome counts alongside latency; separate expired/rejected work from useful throughput. Keep a small-policy case where indexing may not pay back. Keep the original burst and timeout triggers as regressions. Explain one optimization you rejected.

Release decision: proposed change, correctness evidence, effect on successful traffic, retained capacity, remaining limitation, and one metric that would stop a staged rollout. This exercise does not deploy a service.
