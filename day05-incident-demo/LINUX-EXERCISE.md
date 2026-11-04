# Linux task — explain a stuck async service using native evidence

**Slides 24–25 · 10 minutes · work in pairs.** Produce a native Linux thread snapshot and explain it alongside the application's lock events. No Docker is needed.

## 1. Prerequisites

Use your existing course clone on native Linux. You need Python 3, the pinned Rust toolchain and GDB:

```sh
python3 --version
cargo +nightly-2026-07-22 --version
gdb --version
```

If GDB is missing, install your distribution's package (for example `sudo apt install gdb` on Ubuntu/Debian or `sudo dnf install gdb` on Fedora). If dependencies are not cached, fetch them once:

```sh
cargo +nightly-2026-07-22 fetch --manifest-path day05-incident-demo/Cargo.toml --locked
```

## 2. Reproduce, then capture

Run from the repository root:

```sh
python3 day05-incident-demo/run.py deadlock
python3 day05-incident-demo/run.py evidence
python3 day05-incident-demo/run.py report
```

The first run compares the original lock cycle with the repaired ordering over real localhost TCP. The evidence command uses GDB to **launch its own dedicated process**, sets a breakpoint at `classroom_capture_ready`, and runs until the lock cycle is established. It executes `info threads` and `thread apply all bt`, then terminates only that dedicated process. It does not attach to your existing editor/server or change system-wide `ptrace_scope`.

This works with the usual parent-to-child debugging policy. If your managed machine prohibits debugging even a launched child, keep `gdb-capture.log` and report the restriction. Do not disable security policy or run the course application as root to bypass it.

## 3. Open these files

All are under `day05-incident-demo/evidence/`:

| File | What to show |
|---|---|
| `sample-child.log` | Task A/B acquired-lock and next-lock events; `SAMPLE_READY` and PID. |
| `native-sample.txt` | Linux GDB thread list and all-thread backtraces from one stopped instant. |
| `gdb-capture.log` | Complete debugger transcript, including failures if capture is denied. |
| `evidence.json` | UTC timestamp, platform, PID, source/binary hashes and debugger command. |
| `INCIDENT.md` | Generated handoff linking the saved raw runs and their limits. |

## 4. Explain what you see

1. Match the `SAMPLE_READY` PID to the capture and metadata.
2. Complete this table from the log:

   | Task | Lock already held | Lock requested next | Who holds that lock? |
   |---|---|---|---|
   | A | | | |
   | B | | | |

3. Find the GDB `info threads` table. The `*` marks the current debugger thread. `LWP` identifies an OS thread, not an async task.
4. Find each thread's backtrace. `#0` is its innermost frame; increasing frame numbers move towards its callers. **These numbers are stack depths, not sample counts.**
5. The thread at `classroom_capture_ready` was deliberately stopped by our breakpoint. Do not diagnose that marker as the deadlock.
6. Other worker stacks may contain `futex`, `pthread_cond_wait`, `epoll_wait`, `epoll_pwait`, or Tokio park/runtime frames. Names vary with libc, architecture and installed symbols. Explain why waiting workers alone do not prove deadlock.
7. Use the lock events and source to explain the cycle. Suspended Rust futures need not appear as dedicated native thread call stacks.

## 5. Submit a short handoff

Include `sample-child.log`, `native-sample.txt`, `evidence.json` and five sentences:

- What request failed before repair, and what happened after?
- What does each task hold and request?
- What did the native stacks actually show?
- Why does consistent lock ordering prevent this particular cycle?
- What cannot be concluded from this capture alone?

This is a **GDB backtrace snapshot**, not a statistical CPU profile, Linux core dump, or tokio-console session. The instructor's Mac uses `sample` repeatedly over an interval; Linux stops the process at a known marker once. Do not compare macOS sample counts with Linux `#0`, `#1` frame numbers.

References: [GDB thread inspection](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Threads.html), [GDB batch mode](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Mode-Options.html), [Linux Yama parent/child debugging policy](https://www.kernel.org/doc/html/latest/admin-guide/LSM/Yama.html).
