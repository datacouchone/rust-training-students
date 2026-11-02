# Module 10 — production review

Review the remaining-day gateway changes in pairs. For each proposal give a release decision, the relevant invariant, one counterexample, and the regression evidence needed.

1. Admission uses separate atomic load/store operations because a short stress test did not overbook.
2. A deadline wraps a spawned task's JoinHandle; the author assumes returning a timeout means work has stopped.
3. A cancellation guard is created inside an async function body; the wrapper may be dropped before first poll.
4. A parser keeps partial-frame progress in local future variables while callers repeatedly apply timeouts.
5. Accounting increments the accepted count before checking whether the byte total overflows.
6. An optimization wins on eight tenants; production will have thousands and an unbounded per-tenant map.

Deliver a one-page release record: customer impact, acceptance/regression results, resource limits, uncertain side effects and retry policy, canary metric, rollback trigger and missing production evidence. Sources are in SOURCES.md. Local tests and Miri are evidence for specific executions, not a general proof.
