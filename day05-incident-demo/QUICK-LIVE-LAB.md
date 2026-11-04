# Quick live lab: Tokio Console and a real Linux core

Day 5 slide 31. Instructor walkthrough, approximately 6–8 minutes **after setup**.
Run commands from the participant repository root. No exercise source replacement.

## Prepare once, before presenting

Docker must be running. Linux participants can use their native Docker engine.
The first image build and console installation take several minutes and need network access.

```sh
cargo +stable install tokio-console --locked --version 0.1.13 --force -j 1
python3 day05-incident-demo/live_lab.py prepare-console
python3 day05-incident-demo/live_lab.py build-core
```

The console server uses the existing lab implementation, with a separate loopback-only launcher and separate build directory. Instrumentation requires both `--cfg tokio_unstable` and Tokio's `tracing` feature. The Docker image uses the pinned Linux toolchain, release debug information, and GDB.

Use the pinned viewer 0.1.13 with this lab's console-subscriber 0.4.1. Viewer 0.1.14 requires a newer state-streaming API and otherwise keeps reconnecting.

## 1. Observe real async tasks

```sh
python3 day05-incident-demo/live_lab.py console
```

The viewer connects to the actual instrumented lab server. After two seconds the runner sends 12 length-prefixed METRIC frames over TCP. Each request gets OK but also creates a deliberately unfinished task.

Say: “A successful response does not mean all work created by that request has ended. Watch the sleeping tasks remain.”

Use arrows and Enter to inspect a task; Esc returns to the list. Locate tasks spawned at `labs/lab9-core-dump/starter/src/server.rs:129`. There are other runtime tasks: the **overall task count is not expected to equal 12**. An hour-long sleep uses little CPU; it still retains a future and its state.

Show code:

| File / line | Explain |
|---|---|
| `labs/lab9-core-dump/starter/src/tracing_init.rs:19` | Installs tracing and, when enabled, the console subscriber. |
| `labs/lab9-core-dump/starter/src/server.rs:128` | Spawns a task which loops forever, sleeping each iteration. |
| `labs/lab9-core-dump/starter/src/server.rs:91` | The TaskLeak fault creates one such task for each processed frame. |
| `labs/lab9-core-dump/starter/src/bin/live-console.rs:7` | Starts the real lab server on a local-only listener. |

**Press q to leave the viewer before running another terminal command.** The server keeps running so the task state remains inspectable. Reopening the viewer sends another 12 bounded requests.

## 2. Capture and reopen a real Linux core

```sh
python3 day05-incident-demo/live_lab.py core
```

This starts a **separate Linux server** with the deadlock fault. The script waits for both second-lock-attempt logs, confirms a real request times out, and uses GDB to save `core.deadlock`. It stops that server, copies the core and its matching Linux executable to the host, and reopens those files in a separate GDB process.

Say: “This is saved process memory and register state. The original server has stopped, but we can still inspect its native threads.”

Read the final `REAL CORE REOPENED` line, core byte count and binary SHA-256. Inspect `info threads` and the `#0`, `#1` stack frames. Frame numbers are stack depth, not sample counts.

Show `labs/lab9-core-dump/starter/src/server.rs:149`: task A holds metrics and waits for registry; task B holds registry and waits for metrics. Correlate the capture's server log with that source. Tokio suspends futures: **parked worker stacks alone do not reconstruct the async lock ownership or prove this cycle**.

This is a debugger-generated snapshot of a hung process, not a kernel-generated dump from a crash. Both are core files, but the trigger differs.

Files are saved under `day05-incident-demo/evidence/live-lab/`:

- `core-capture/cores/core.deadlock`: real Linux core file.
- `matching-linux-binary`: exact executable from the capture container.
- `core-capture/server.log`: fault and request context.
- `core-backtraces.txt`: GDB output from reopening the saved files.
- `core-metadata.json`: file size and executable hash.

The capture container is retained as `rust-day5-live-core`. Repeated `core` runs inspect the existing evidence; they do not overwrite it.

## 3. Reinspect and finish

```sh
python3 day05-incident-demo/live_lab.py inspect
python3 day05-incident-demo/live_lab.py stop
```

Stopping the dedicated console server ends its intentionally leaked tasks. The saved core remains available. The observation does not repair the original lab: use the existing demonstrated fixes and regressions to check the proposed repair.

| Tool | Answers | Limit |
|---|---|---|
| Tokio Console | Which instrumented async tasks are live, busy or waiting? | Requires a running, instrumented process; task age alone does not prove a leak. |
| GDB + core + matching binary | What native process state was saved at capture time? | No subsequent execution; native threads are not a complete automatic map of suspended async futures. |
