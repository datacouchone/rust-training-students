# Day 5 — runnable incident walkthrough (slides 23–30)

Run from the participant repository root:

```sh
python3 day05-incident-demo/run.py all
```

This independent miniature service reproduces the mechanisms reviewed in `labs/lab9-core-dump/starter/src/server.rs`. It uses actual TCP on loopback with ephemeral ports, a bounded client deadline and two Tokio worker threads. Original lab attempts are unchanged. Both named BEFORE and AFTER functions are present in `src/main.rs`; the runner executes both, without replacing source.

| Slide | Command | Show and explain |
|---|---|---|
| 23 | `python3 day05-incident-demo/run.py all` | All three real TCP fault/repair pairs. |
| 23 | `python3 day05-incident-demo/run.py deadlock` | Two acquired locks plus opposite waits; timeout before, OK after consistent ordering. |
| 23 | `python3 day05-incident-demo/run.py leak` | Successful replies leave 1, 2, 3 tasks alive before; finite joined children leave zero after. |
| 23 | `python3 day05-incident-demo/run.py panic` | Malformed input panics one connection task before; repaired parser returns ERR. A second valid connection succeeds. |
| 24 | `python3 day05-incident-demo/run.py evidence` | Native Linux GDB backtraces or macOS thread sample, plus the child's lock events. Linux task: [LINUX-EXERCISE.md](LINUX-EXERCISE.md). |
| 25 | `python3 day05-incident-demo/run.py report` | Preserve and review the evidence files, timestamps and source/binary identities. |
| 26 | `python3 day05-incident-demo/run.py test` | Five executable regression checks. The injected BEFORE panic is expected; read the final test results. |
| 27 | `python3 day05-incident-demo/run.py retry` | Real TCP commit with a deliberately discarded ACK; retry and conflicting content before/after an in-memory receipt ledger. |
| 28 | `python3 day05-incident-demo/run.py review` | Rerun all faults and calculate a comparison table from their actual output. |
| 29 | `python3 day05-incident-demo/run.py rollout` | Execute the decision function on five labelled fixture windows. These are not production measurements. |
| 30 | `python3 day05-incident-demo/run.py report` | Generate an incident handoff backed by saved raw evidence. |

## What differs from the original lab

- The lock cycle uses a barrier rather than a timing-dependent sleep, making the trigger deterministic.
- The leak demo retains task handles in a supervisor solely so it can clean up injected leaks after measuring them. The original lab discards the handle. The repaired path joins finite child work before replying.
- The parser uses a numeric payload to isolate Result versus panic. The original lab parses METRIC records.
- The runner supervises the panicking connection task to expose its JoinError. The original lab drops its JoinHandle.
- Each comparison uses fresh state. The repaired lock-order tasks finish before its probe request. This demonstrates recovery from that mechanism, not a throughput comparison.

## Reading the evidence

`evidence/` contains actual raw output and run metadata. Native capture writes `native-sample.txt` and `sample-child.log`. On Linux, GDB launches the child, stops at the ready marker and records all thread backtraces; the complete transcript is `gdb-capture.log`. On macOS, `sample` collects stacks over one second. Neither capture is a Linux core dump. Linux backtrace frame numbers are not macOS sample counts. Parked threads alone do not prove deadlock: correlate the sample with the held-lock and wait events.

The task counts are application lifetime guards, **not a tokio-console session**. A sleeping future consumes resources without continuously consuming CPU. The original console configuration remains a supplemental code highlight on slide 23; its client must be installed and its build instrumentation enabled for a separate console session.

The runner bounds commands and cleans up its own tasks/child process. It does not start Docker or change the original lab, other days or final challenge. The replay receipts are process-local and in memory. Passing these scenarios does not establish production SLOs, all possible schedules or crash durability.
