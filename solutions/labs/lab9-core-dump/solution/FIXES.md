# FIXES.md — instructor reference

Root cause, evidence trail, and fix for each of the three faults in
`src/server.rs`. The evidence below is real output captured from this
exact codebase (`RUST_LOG=info`, healthy defaults unless noted); command
lines are included so you can reproduce every line yourself before class.

The point of this document isn't the fixes themselves -- they're each one
line. The point is the *method*: reproduce, observe with the cheapest tool
first, escalate only when you need to, then name the mechanism precisely
enough that the fix is obvious. Walk attendees through the evidence
sections, not straight to "Fix".

---

## Fault 1: `deadlock`

### Reproduce

```bash
export CARGO_TARGET_DIR="/Users/gojo/Rust training/labs/target/lab9"
FAULT=deadlock RUST_LOG=info cargo run -p lab9-core-dump-solution --bin lab9-core-dump-solution
# in another shell:
python3 docker/client.py 9000 "METRIC cpu.load 0.42"
```

The client hangs forever. No `OK`, no `ERR`, no error, no crash. The
server process is still running (`ps` shows it) -- it's *wedged*, not
dead. This distinction matters: a health check hitting `/health` on a
separate lightweight endpoint might even report healthy while every
metrics connection hangs.

### Evidence trail

**Step 1 — logs (cheapest tool, check first).** The server logs the last
thing each background task did before it went silent:

```
INFO lab9 metrics server listening fault=Some(Deadlock) local_addr=0.0.0.0:9000
WARN FAULT=deadlock active: arming AB-BA mutex deadlock pair now
INFO deadlock_task_b: task_b: locked registry; sleeping before locking metrics
INFO deadlock_task_a: task_a: locked metrics; sleeping before locking registry
INFO deadlock_task_b: task_b: attempting metrics lock
INFO deadlock_task_a: task_a: attempting registry lock
INFO accepted connection conn_id=1 peer_addr=127.0.0.1:61557
INFO handle_connection{conn_id=1}: connection opened
```

Then nothing, ever again -- not even `frame read error` or a timeout log
from the accepted connection. Two things jump out:

1. `task_a` says "attempting registry lock" and never logs anything after.
   `task_b` says "attempting metrics lock" and never logs anything after.
   Read literally: task_a wants the lock task_b already holds, and
   task_b wants the lock task_a already holds.
2. `conn_id=1`'s connection got as far as "connection opened" and then
   *also* went silent -- before it could even log a frame. Look at
   `handle_connection`: the very first thing it does is
   `state.registry.lock().await` to register the connection. That's the
   same `registry` mutex `task_b` is holding. The real connection isn't
   broken by a *different* bug -- it's the third victim of the same one.

This one span-per-task naming choice (`deadlock_task_a` /
`deadlock_task_b`, and the connection span carrying `conn_id`) is what
makes the log readable at all. Without it you'd have two anonymous
`tokio-rt-worker` lines saying "locked X" and no way to tell which task
owns which guard.

**Step 2 — tokio-console (next cheapest: live, no restart needed).**

```bash
RUSTFLAGS="--cfg tokio_unstable" TOKIO_CONSOLE=1 RUST_LOG=info FAULT=deadlock \
  cargo run -p lab9-core-dump-solution --bin lab9-core-dump-solution
# separate shell:
tokio-console
```

In the `tasks` view, `deadlock_task_a` and `deadlock_task_b` sit in
`Idle`/parked state with `Busy` time frozen and a nonzero, growing "total"
(wall) time -- they aren't running, they aren't scheduled, they're
blocked waiting to be woken by a lock release that will never come. The
`resources` view shows the two `tokio::sync::Mutex` instances each with
one waiter queued. This is the fastest way to confirm "blocked on a lock",
without attaching a debugger.

**Step 3 — core dump (when you need the full picture: exact wait chain,
native stack, works even if tracing was misconfigured or the process is
too far gone to talk to tokio-console).** See `../docker/analyze.md` for
the full gdb walkthrough. The short version: `info threads` shows two
tokio worker OS threads parked in the mutex's internal futex/condvar wait
inside `tokio::sync::Mutex::lock`, and their *tracing span IDs* (visible
via `poll` frame arguments in the async state machine, or correlated by
timestamp against the log above) map back to `deadlock_task_a` and
`deadlock_task_b`. The core dump doesn't show you the Rust-level "task",
because there isn't a task on the native stack to show (see below) -- it
shows you an OS thread parked in a syscall, and you have to bring the log
timestamps with you to say *which* task that thread was running when it
parked.

### Root cause

`arm_deadlock` in `src/server.rs` spawns two tasks that take the same two
`tokio::sync::Mutex`es (`metrics`, `registry`) in **opposite order**, and
each holds its first guard **across an `.await` point** before requesting
the second:

```rust
// task_a
let _guard_a = a1.lock().await;         // locks metrics
tokio::time::sleep(...).await;          // <-- guard_a still held here
let _guard_b = b1.lock().await;         // wants registry

// task_b
let _guard_b = b2.lock().await;         // locks registry
tokio::time::sleep(...).await;          // <-- guard_b still held here
let _guard_a = a2.lock().await;         // wants metrics
```

Once task_a has `metrics` and task_b has `registry`, task_a's request for
`registry` and task_b's request for `metrics` can never both be granted.
Every other user of either mutex (every `record_metric` call, every
connection's registry push/pop) queues up behind them and never runs.

**Why async makes this easy to write by accident:** in synchronous Rust,
holding a `std::sync::MutexGuard` across a blocking call is unusual and
`clippy::await_holding_lock` doesn't even apply. In async code, holding a
guard across an `.await` is *extremely* easy to do without noticing --
the guard is just a local variable that happens to still be in scope, and
nothing about the syntax calls out "you are blocking every other task
that wants this lock for the entire duration of this await". `tokio::sync::Mutex`
guards are `!Send`-free and perfectly happy to be held across awaits,
which is sometimes exactly what you want -- and sometimes exactly this
bug.

### Fix

Never hold a lock guard across an `.await` when more than one lock is
involved -- either don't await while holding it (copy out what you need,
drop the guard, then await), or establish and document a single global
lock order and never violate it. For this code, the guard is only held
across the sleep for the sake of demonstrating the race window; the real
fix is structural:

```rust
// Fixed: acquire, use, and drop each lock before acquiring the next.
// No guard is ever live across an .await.
async fn safe_two_lock_operation(state: &AppState) {
    let snapshot = {
        let guard = state.metrics.lock().await;
        guard.clone() // or whatever you actually need from it
    }; // guard dropped here, before the next lock or any await

    let mut registry = state.registry.lock().await;
    registry.push(/* ... using snapshot ... */);
} // guard dropped here
```

If a task genuinely needs both locks held simultaneously (rare), fix the
*order* instead: pick one global order (e.g. always `metrics` before
`registry`) and enforce it everywhere, including in code review and,
ideally, with a lint or a wrapper type that only exposes "lock both, in
order" as an API.

---

## Fault 2: `task_leak`

### Reproduce

```bash
FAULT=task_leak RUST_LOG=info cargo run -p lab9-core-dump-solution --bin lab9-core-dump-solution
python3 docker/client.py 9000 "METRIC a 1" "METRIC b 2" "METRIC c 3"
```

Every frame is answered `OK` -- functionally, nothing looks wrong from
the client's point of view. This is what makes it dangerous: it's not a
crash or a hang, it's a slow leak that only shows up under sustained load.

### Evidence trail

**Step 1 — logs.** Grep for the leaked-task span:

```
$ grep leaked_task server.log | wc -l
3
INFO handle_connection{conn_id=1}:process_frame{conn_id=1 frame_no=1}:leaked_task{conn_id=1 frame_no=1}: leaked task started; it will never terminate
INFO handle_connection{conn_id=1}:process_frame{conn_id=1 frame_no=2}:leaked_task{conn_id=1 frame_no=2}: leaked task started; it will never terminate
INFO handle_connection{conn_id=1}:process_frame{conn_id=1 frame_no=3}:leaked_task{conn_id=1 frame_no=3}: leaked task started; it will never terminate
```

Three frames in, three `leaked_task` spans logged, and -- notice what's
*missing* -- no corresponding "finished" or "exiting" line for any of
them, ever, no matter how long you wait. Compare to `handle_connection`,
which reliably logs "connection closed" on the way out. One line of log
output per frame, with no matching completion, is the signature of an
unbounded spawn.

**Step 2 — tokio-console.** Open the `tasks` view and watch the total
task count while sending frames: it climbs by exactly one non-leaked task
per connection, plus **one additional task per frame** while
`FAULT=task_leak` is active, and none of the `leaked_task`-named entries
ever transition out of the running/idle state to `Completed`. Sort by
"total time" (or just watch the count) during a short load burst and the
slope is a straight, unbounded line -- compare that to the healthy run,
where task count plateaus at "one per open connection".

**Step 3 — core dump (only if you need it).** By the time you'd reach for
a core dump, the smoking gun is usually already in tokio-console. A dump
would show hundreds/thousands of near-identical OS-thread-adjacent async
task states all parked in the same `tokio::time::sleep` inside
`spawn_leaked_task`'s loop -- useful mainly to confirm *scale* (how many)
at the moment of an OOM, not to find the *mechanism* (tokio-console
already told you that live).

### Root cause

`spawn_leaked_task` in `src/server.rs`, called once per frame from
`process_frame` when `FAULT=task_leak`:

```rust
fn spawn_leaked_task(conn_id: u64, frame_no: u64) {
    tokio::spawn(
        async move {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        }
        .instrument(...),
    );
}
```

The returned `JoinHandle` is dropped immediately (never bound to a
variable) -- which is fine, that alone doesn't leak the task, tokio tasks
run independently of their handle. The actual bug is that the task's own
body is an infinite loop with no exit condition, no cancellation token, no
timeout, and nothing external (a channel close, a shutdown signal) that
could ever cause it to return. A task that never resolves and is never
aborted stays allocated on the tokio runtime forever. One per frame, with
sustained traffic, is unbounded growth in task count and the associated
memory (each task's future, plus its tracing span data, plus scheduler
bookkeeping).

### Fix

Two independent problems, fix both:

1. **Give the task a reason to stop.** Nothing in this server's design
   actually needs a background task per frame; this is here purely to
   demonstrate the fault. If you did need per-request background work,
   it must have a termination condition -- a bounded number of iterations,
   a deadline, or a cancellation signal:

   ```rust
   fn spawn_bounded_task(conn_id: u64, frame_no: u64, mut shutdown: tokio::sync::watch::Receiver<bool>) {
       tokio::spawn(async move {
           tokio::select! {
               _ = shutdown.changed() => {}
               _ = tokio::time::sleep(Duration::from_secs(30)) => {}
           }
       });
   }
   ```

2. **Track what you spawn.** Even a bounded task is a liability if nothing
   owns it -- keep spawned `JoinHandle`s in a `JoinSet` (or an
   `Arc<Mutex<Vec<JoinHandle<_>>>>` at minimum) so the server can observe
   how many background tasks are outstanding and abort them on shutdown or
   connection close, instead of relying on every task to self-terminate
   correctly.

The best fix for *this* server is simpler still: delete the fault-only
`spawn_leaked_task` call from the request path entirely. It's not doing
real work; it exists to demonstrate the shape of the bug.

---

## Fault 3: `panic`

### Reproduce

```bash
FAULT=panic RUST_LOG=info cargo run -p lab9-core-dump-solution --bin lab9-core-dump-solution
python3 docker/client.py 9000 "METRIC cpu.load 0.42" "GARBAGE"
```

Real captured output:

```
--> 'METRIC cpu.load 0.42'
<-- b'OK'
--> 'GARBAGE'
<-- EOF (connection closed)
```

and, in the server's own log, sandwiched inside the tokio panic hook's
plain-text output (not a `tracing` event -- panics bypass your subscriber
unless you install a panic hook that routes through it):

```
INFO handle_connection{conn_id=1}: connection opened

thread 'tokio-rt-worker' panicked at lab9-core-dump/solution/src/server.rs:102:29:
called `Result::unwrap()` on an `Err` value: UnknownCommand("GARBAGE")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

A *second* client connecting right after is served completely normally --
`conn_id=2` opens, sends a frame, gets `OK`. The server did not crash.

### Evidence trail

**Step 1 — logs are the whole story here.** The panic message alone gives
you the file, line, and exact value that broke: `server.rs:102:29`,
`UnknownCommand("GARBAGE")`. The surrounding `tracing` spans
(`handle_connection{conn_id=1}`) tell you *which connection* -- cross-
reference the log lines immediately before the panic to also recover
`frame_no` from the `process_frame` span that would have logged, had it
gotten the chance. (It didn't, in this trace, because the panic happens
before any `warn!` in `process_frame`'s error branch -- that's itself a
clue that the crash is in the *parse-and-decide* step, not somewhere
later.)

**Step 2 — the "why didn't the server die" question, answered by
behavior, not logs.** `tokio::spawn` wraps each connection's future in a
task that tokio polls via `catch_unwind`-protected machinery: a panic
inside a spawned task unwinds *that task* and turns its `JoinHandle` into
a `JoinError::is_panic()`, but does not propagate to the task that called
`tokio::spawn` (`run`'s accept loop) or to any other task. That's why
`conn_id=1`'s client sees a silent EOF (their task's future was dropped
mid-unwind, so the socket just closes) while the accept loop and every
other connection carry on untouched. This is worth stating explicitly to
attendees: **tokio isolates panics per task by default**, which is a
double-edged property -- it keeps one bad request from taking down the
server, but it also means a per-request panic can go unnoticed in
production for a long time if nothing is watching for the `EOF`-with-no-
response pattern or for `JoinError::is_panic()` on the handle.
tokio-console surfaces this too: the panicked task's row disappears from
`tasks` (it completed, via panic) rather than showing an error state --
so for this fault, logs are actually your best signal, not the console.

**Step 3 — core dump.** Not useful for this fault by the time you'd take
one: by the time you could `SIGABRT` and inspect a dump, the panicking
task has already unwound and its stack is gone. If you want a *native*
core dump captured *at the moment of panic*, you'd set
`panic = "abort"` for that build (which changes the isolation behavior
above -- an abort takes the whole process down) or run under a debugger
with a breakpoint on `rust_panic` / `__rust_start_panic`. That's a
legitimate technique but a different exercise from the AB-BA deadlock
core dump this lab's `docker/` walkthrough focuses on.

### Root cause

`process_frame` in `src/server.rs`, when `FAULT=panic`:

```rust
if state.fault == Some(Fault::Panic) {
    let metric = parsed.unwrap(); // <-- panics on any Err
    record_metric(state, metric).await;
    "OK".to_string()
} else {
    match parsed {
        Ok(metric) => { ... "OK" }
        Err(e) => { ... format!("ERR {e}") }  // <-- the correct behavior
    }
}
```

`parse_metric_frame` (see `src/frame.rs`) is a total function -- it always
returns `Result<Metric, FrameError>` and never panics, for any input
bytes, including empty, non-UTF-8, or garbage. The bug is entirely in how
the *caller* handles that `Result`: `.unwrap()` converts "this input was
malformed" (an expected, everyday outcome for a server that accepts
untrusted input) into "this program has an unrecoverable invariant
violation" (what panics are supposed to mean). Those are not the same
thing, and conflating them is precisely what turns "a client sent a typo'd
metric name" into an incident.

### Fix

Match on the `Result` and answer `ERR <reason>` instead of unwrapping --
which is exactly what the healthy (`else`) branch already does two lines
below. The fix is to delete the fault-only `if` branch and always take
that path:

```rust
async fn process_frame(conn_id: u64, frame_no: u64, bytes: &[u8], state: &AppState) -> String {
    match parse_metric_frame(bytes) {
        Ok(metric) => {
            record_metric(state, metric).await;
            "OK".to_string()
        }
        Err(e) => {
            tracing::warn!(error = %e, "rejecting malformed frame");
            format!("ERR {e}")
        }
    }
}
```

More generally: `.unwrap()`/`.expect()` on anything derived from network
input is a defect, full stop -- reserve them for invariants your own code
established (e.g. "this `Vec` I just pushed to is non-empty"), never for
"the client sent well-formed data". A `#[deny(clippy::unwrap_used)]` (or
at least `warn`) on request-handling modules, enforced in CI, would have
caught this at compile time rather than in production.
