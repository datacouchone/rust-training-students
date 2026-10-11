# analyze.md — reading the `deadlock` core dump with gdb

Run the Docker commands in the [lab README](../README.md). The entrypoint
captures a live hung Linux process with GDB, then opens a Linux shell. The
container is named `lab9-crash` and is retained after exit. Copy its snapshot,
exact matching binary and logs before removing it. This is a live snapshot,
not a signal-generated crash dump; no global core-dump settings are changed.

> **A note on the output shown here.** The gdb commands below are exactly
> what you'll type. The *output* shown after each one is a realistic,
> structurally-accurate example of what you'll see -- exact thread/LWP
> numbers, addresses, and PIDs will differ on every run and are not
> meaningful to memorize. What *is* stable, and what you should be
> looking for, is called out after each block.

---

## 0. Load the core

```bash
gdb /workspace/target/release/lab9-core-dump-starter /tmp/lab9-crash/cores/core.deadlock
```

(`rust-gdb` -- same binary, plus Rust pretty-printers for `Vec`/`String`/
`Option`/enums -- is worth using if it's on `$PATH`; plain `gdb` works
fine for everything in this walkthrough, it just prints structs a bit
more raw.)

GDB loads the saved process state. Because this snapshot came from a live
process, do not expect SIGABRT to be reported. Addresses, thread counts and
native frames vary by architecture and by the instant captured.

---

## 1. `info threads` -- find the tokio worker threads

```
(gdb) info threads
```

```
  Id   Target Id                             Frame
* 1    Thread 0x7f2a1c0a0740 (LWP 1)         futex_wait (...) at ./nptl/futex-internal.c:57
  2    Thread 0x7f2a1994f6c0 (LWP 8) "tokio-rt-worker"  futex_wait (...) at ./nptl/futex-internal.c:57
  3    Thread 0x7f2a1914e6c0 (LWP 9) "tokio-rt-worker"  futex_wait (...) at ./nptl/futex-internal.c:57
  4    Thread 0x7f2a1894d6c0 (LWP 10) "tokio-rt-worker" futex_wait (...) at ./nptl/futex-internal.c:57
  5    Thread 0x7f2a1814c6c0 (LWP 11) "tokio-rt-worker" futex_wait (...) at ./nptl/futex-internal.c:57
```

- **Thread 1** is the process's main OS thread -- the one that ran
  `#[tokio::main]`'s generated `Runtime::block_on`. It doesn't carry the
  `tokio-rt-worker` name (that name is only set on threads the runtime's
  thread-pool builder spawns itself); you identify it by being LWP 1 /
  thread ID matching the PID.
- **Threads 2-N** are named `"tokio-rt-worker"` -- that string comes
  straight from tokio's scheduler (`Builder::new_multi_thread`'s default
  `thread_name`, unchanged in this lab), so it's the reliable way to pick
  the pool workers out of a mixed thread list in any tokio process, not
  just this one. By default there's one per logical CPU.
- If `TOKIO_CONSOLE=1` had been set, you'd also see a thread named
  `"console_subscriber"` -- not present here since `run-crash.sh` doesn't
  set it (see `../README.md`'s tokio-console section for that flow
  separately, while the process is still alive).

**Already useful, before reading a single backtrace:** every worker
thread is inside `futex_wait`, not spinning in user code and not gone.
This snapshot is consistent with idle workers, but cannot establish the
process's behavior over time. A periodic busy loop or pending I/O also needs
to be checked against logs, live task state and repeated observations. That's consistent with "no
worker has any runnable task to poll" -- which is exactly what you'd
expect if every task that exists is blocked waiting on a waker that will
never fire.

---

## 2. `thread apply all bt` -- and why it *won't* show `arm_deadlock`

```
(gdb) thread apply all bt
```

```
Thread 4 (Thread 0x7f2a1894d6c0 (LWP 10) "tokio-rt-worker"):
#0  futex_wait (...) at ./nptl/futex-internal.c:57
#1  __GI___pthread_cond_wait (...) at ./nptl/pthread_cond_wait.c:503
#2  0x... in std::sys::pal::unix::locks::pthread_condvar::Condvar::wait (...)
#3  0x... in std::sync::condvar::Condvar::wait (...)
#4  0x... in tokio::runtime::scheduler::multi_thread::park::Inner::park (...)
#5  0x... in tokio::runtime::scheduler::multi_thread::worker::Context::park (...)
#6  0x... in tokio::runtime::scheduler::multi_thread::worker::run (...)
#7  0x... in tokio::runtime::task::harness::Harness<T,S>::poll (...)
#8  0x... in std::sys::backtrace::__rust_begin_short_backtrace (...)
#9  0x... in core::ops::function::FnOnce::call_once{{vtable.shim}} (...)
#10 0x... in std::sys::pal::unix::thread::Thread::new::thread_start (...)
#11 0x... in start_thread (...) at ./nptl/pthread_create.c:447
```

Every worker thread's backtrace looks like this -- and this is the
central, slightly counterintuitive lesson of debugging async Rust from a
core dump: **you will not see `arm_deadlock`, `handle_connection`,
`record_metric`, or any `.await` point from our code anywhere in these
native stacks.**

**Why:** an OS thread's call stack only reflects frames that are
currently, actually, on that thread's native stack. `tokio::spawn`ed
futures are not run on dedicated stacks -- each one compiles down to a
state machine (an enum whose variants are "which `.await` point am I
paused at, and what are my live local variables at that point") that
lives in a heap allocation owned by the runtime's task list. A worker
thread's native stack only contains a **task's** frames *while that
worker is actively polling it* -- frame #7 (`Harness::poll`) above is
the generic "poll whatever task I'm handed" entry point, not a
per-task-shaped frame. The instant a task returns `Poll::Pending` (which
is what happens the moment `.lock().await` can't immediately acquire the
mutex), its state machine is parked on the heap, the worker's native
stack unwinds straight back through `Harness::poll` with nothing of the
task's own call chain left behind, and the worker goes look for other
work -- finds none, in this case -- and parks itself (frames #0-#6
above). This is "stack fragmentation": the *logical* call chain
(`main` → `run` → `handle_connection` → `record_metric` →
`Mutex::lock`) is real, but it is never simultaneously present as one
contiguous native stack the way a classic synchronous thread's would be.
It's fragmented across heap-allocated future state, and reassembling it
from raw memory alone (without symbols specifically for tokio's internal
task representation, which isn't a stable, documented layout) is
impractical -- which is exactly why `tracing` spans and `tokio-console`
exist: they let the *program itself* narrate which logical async task is
doing what, in a form you can inspect without needing to solve this
heap-archaeology problem by hand.

A classic **synchronous** `std::sync::Mutex` AB-BA deadlock would *not*
have this problem -- `thread apply all bt` on that core would show
`pthread_mutex_lock` called directly from your function, on that
thread's own native stack, because a blocked OS thread's call chain
really is sitting right there. If you want to see that contrast
concretely: this is also why the `panic` fault (a genuine, ordinary Rust
panic unwind) *would* show recognizable frames if you captured it with a
native debugger at the right moment -- unwinding walks real stack frames.
An async task parked mid-`.await` has no stack frames left to walk.

---

## 3. Reconnecting the native evidence to the logical one

Since the core dump alone won't name the two stuck tasks, bring
`server.log` with you -- this is the payoff of the tracing spans set up
in `src/server.rs` (`deadlock_task_a`, `deadlock_task_b`,
`handle_connection{conn_id=...}`):

```bash
tail -n 10 /tmp/lab9-crash/server.log
```

```
INFO deadlock_task_b: task_b: locked registry; sleeping before locking metrics
INFO deadlock_task_a: task_a: locked metrics; sleeping before locking registry
INFO deadlock_task_b: task_b: attempting metrics lock
INFO deadlock_task_a: task_a: attempting registry lock
INFO accepted connection conn_id=1 peer_addr=...
INFO handle_connection{conn_id=1}: connection opened
```

Nothing logs after that -- three tasks (`deadlock_task_a`,
`deadlock_task_b`, and `handle_connection{conn_id=1}`) each got as far as
requesting a lock and then produced no further output, ever, no matter
how long the observation window lasted before GDB captured the snapshot. Combined with
step 2's finding (every worker thread is idle, not crashed, not
spinning), you can now state the full picture with confidence *without
ever having seen an async frame in gdb*:

1. Some task holds `metrics` and wants `registry` (`task_a`, per the log).
2. Some task holds `registry` and wants `metrics` (`task_b`, per the log).
3. A third task (`conn_id=1`'s connection handler) is *also* now stuck --
   not because it has its own bug, but because it too needs `registry`,
   which `task_b` is holding.
4. No worker thread is doing any work, because every task in the system
   is transitively blocked on this cycle -- which is precisely why the
   whole server is wedged, not just the two originating tasks.

That's the deadlock, fully diagnosed, cross-referencing the core dump's
"nobody is crashed, nobody is spinning, everybody is parked" with the
log's "here is the exact lock each stuck task was last requesting, and
in what order it acquired the first one."

---

## 4. (Optional, advanced) looking for the mutex's waiter state directly

If you want to go one level deeper than "trust the logs": `tokio::sync::Mutex`
is implemented on top of a semaphore (`tokio::sync::batch_semaphore`). You
can, in principle, locate a live `Mutex<HashMap<String, f64>>` or
`Mutex<Vec<u64>>` in the heap (e.g. via `info proc mappings` to find the
heap range, then scanning for the semaphore's internal permit counter)
and inspect its waiter count. In practice this requires knowing tokio's
internal (unstable, version-specific) struct layout, isn't something
`rustc`'s debug info makes convenient even with pretty-printers, and is
rarely worth it once the logs already tell the same story. Mentioned here
so you know the theoretical floor exists, not because you should reach
for it during a real incident -- reach for tracing and tokio-console
first, every time; this is the fallback for when *those* aren't
available (e.g., a crash in a build that shipped without them).

---

## Troubleshooting

**No core file or attach denied.**

Read `/tmp/lab9-crash/capture.log`. Run the container with `--cap-add=SYS_PTRACE`
and `--security-opt seccomp=unconfined`. Docker isolation policy may still
forbid attachment; rehearse on an allowed dedicated Linux VM in that case.
Do not change host/VM `core_pattern` as a workaround. A one-thread dump is
not automatically invalid: inspect the actual process and CPU limits.

**gdb shows `??` instead of function names / no source lines.**

- The binary must be the exact one that produced the core (same build,
  not rebuilt in between) and built with debug info. This lab's release
  profile already sets `debug = true` at the workspace level, so a
  standard `cargo build --release` (as the Dockerfile does) should always
  have symbols. If you rebuilt the binary after generating the core, the
  two no longer match -- regenerate one or the other.

**I only see 1-2 `tokio-rt-worker` threads and expected more.**

- Tokio sizes the worker pool to the number of *logical CPUs visible to
  the container* by default, which can be very small under constrained
  Docker resource limits. This doesn't change the walkthrough -- the
  same "all workers parked" observation applies regardless of pool size.
