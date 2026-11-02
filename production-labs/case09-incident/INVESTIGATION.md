# Gateway incident record

The probes emit locally generated synthetic observations. Preserve the exact source version and commands. Avoid reading all acceptance tests as an answer key before recording your first hypotheses.

| Scenario | Command | Expected contract | Actual observation | Hypothesis A | Hypothesis B | Experiment that separates them |
|---|---|---|---|---|---|---|
| 1 | probe 1 | | | | | |
| 2 | probe 2 | | | | | |
| 3 | probe 3 | | | | | |

For each repair record:

- Trigger and minimal reproducible input/sequence.
- Evidence that localizes the mechanism; include the state before/after suspension or rejection.
- Blast radius: one frame, one stream, one tenant, all tenants?
- Proposed change and why it repairs that mechanism.
- Regression command, selected test count and actual outcome.
- A plausible alternative fix rejected, with a counterexample.
- Deployment guard, rollback trigger and an unknown requiring production evidence.

The in-memory probes do not validate real socket load, runtime starvation, TLS, tracing setup or Linux core capture. Use the original server lab for those operational investigations. No Docker startup is part of this task.
