# Investigation Worksheet

You have a working (but faulty) server in `src/`. Your job is not to guess
the bug from reading the source top to bottom -- it's to **reproduce it,
observe it with the tools this module covers, and pin down the exact
mechanism**, the way you would for a production incident where you don't
start with a diff.

Work through the three faults **one at a time**. For each one:

```bash
export CARGO_TARGET_DIR="${PWD}/target/lab9"
FAULT=<deadlock|task_leak|panic> RUST_LOG=info \
  cargo run -p lab9-core-dump-starter --bin lab9-core-dump-starter
```

The server listens on `127.0.0.1:9000` by default (override with
`METRICS_PORT`). Send it frames with the tiny Python client in
`../docker/` or with your own -- see the top-level `README.md`'s "Starter
entry point" section for the wire format and a copy-pasteable client.

Fill in a copy of each table below per fault. Don't skip to the root
cause -- write down what you observe *before* you know why.

---

## 1. Symptoms table

| Fault | What you sent | What you expected | What actually happened | How long until you noticed? |
|---|---|---|---|---|
| deadlock | | | | |
| task_leak | | | | |
| panic | | | | |

---

## 2. Evidence log

For each fault, capture at least one piece of evidence from **each** of
the three tools this module covers, and note what it told you:

### `deadlock`

- [ ] **Structured logs** (`RUST_LOG=info`, or `debug` if you need more):
      which span/task logged last before things went quiet? What was it
      about to do?
- [ ] **tokio-console** (`TOKIO_CONSOLE=1`, requires
      `RUSTFLAGS="--cfg tokio_unstable"` -- see README): which tasks are
      shown as `Running` vs stuck? What are they waiting on?
- [ ] **Core dump** (see `../docker/analyze.md`): once you have a core
      file, which threads in `info threads` are the tokio worker threads?
      What are they blocked on in their backtraces?

Evidence:

```
(paste log lines / tokio-console observations / gdb output here)
```

### `task_leak`

- [ ] **Structured logs**: what pattern do you see in the logs as you send
      more frames? Is there a log line that appears once per frame that
      shouldn't?
- [ ] **tokio-console**: watch the `tasks` view while sending frames. What
      happens to the total task count? Do any tasks ever move to
      `Completed`?
- [ ] Optional: what would this look like in `top`/`ps` (thread count) or
      memory usage over a long soak?

Evidence:

```
(paste observations here)
```

### `panic`

- [ ] **Structured logs**: what does the panic message tell you? Which
      file/line? Which `conn_id`/`frame_no` (from the surrounding spans)?
- [ ] What happens to *that* connection? What happens to *other*,
      concurrent connections? Why the difference (think about how
      `tokio::spawn` isolates panics)?
- [ ] Optional: run with `RUST_BACKTRACE=1` -- what does the full
      backtrace add?

Evidence:

```
(paste observations here)
```

---

## 3. Root-cause template

For each fault, write 2-4 sentences using this shape:

> **Fault:** `<name>`
> **Trigger:** what input/sequence causes it
> **Mechanism:** the precise reason it happens (not just "there's a bug in
> X" -- name the exact ordering, missing check, or invariant violation)
> **Blast radius:** what breaks -- one connection? the whole process? does
> it recover on its own?
> **Fix (your proposal, before you look at any reference material):**
> what would you change, and why does that change break the mechanism you
> identified?

```
Fault: deadlock
Trigger:
Mechanism:
Blast radius:
Fix:
```

```
Fault: task_leak
Trigger:
Mechanism:
Blast radius:
Fix:
```

```
Fault: panic
Trigger:
Mechanism:
Blast radius:
Fix:
```

---

## 4. Self-check

- Can you point to the exact line(s) in `src/server.rs` responsible for
  each fault, and explain in one sentence why that line is wrong?
- For the deadlock: can you draw the lock-order diagram (which task takes
  which lock in which order) from your evidence alone, without opening
  `arm_deadlock` in the source?
- For the leak: could you tell `task_leak` apart from a *slow* task
  (one that's just taking a long time) using only tokio-console? What's
  the tell?
- For the panic: is `unwrap()` on user-controlled input a code-review-time
  bug, a testing-time bug, or a runtime-monitoring-time bug? Does your
  fix change which one it is?
