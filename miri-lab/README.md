# Miri lab: investigate memory and concurrency bugs

A standalone extension for Day 2. No completed earlier lab is required.
Plan: three 50-minute sessions, with a 10-minute break after each. The final
publication experiment is an optional 25–35-minute extension.

## What is Miri?

Miri executes Rust's intermediate representation in an interpreter that checks
operations against Rust's rules. Think of a practice run with an inspector
watching each memory access. Unlike a normal test, it can identify an invalid
access even when the program seems to produce a plausible result.

MIR means **Mid-level Intermediate Representation**. Your Rust code is lowered
into this compiler representation; Miri interprets it rather than running your
normal native executable. It is a development tool, not a runtime component you
ship with your application and not a performance benchmark.

**Undefined behavior (UB)** means the program violates a requirement on which
Rust relies, such as reading freed storage. After UB, a native result is not a
reliable explanation of what the program means. An `unsafe` block makes you
responsible for its requirements; it does not turn the checks off conceptually.

By the end you should be able to:

- Run a focused Miri check and read the first useful diagnostic.
- Distinguish compilation failure, ordinary panic, UB, and an unsupported operation.
- Repair lifetime, initialization and bounds errors.
- Explain a data race versus a logical race, and repair counters with locks/atomics.
- Explain what a passing run establishes and what it leaves untested.

## Files to open

| File | Purpose |
|---|---|
| `examples/01_dangling.rs` | Use after free: address outlives owner |
| `examples/02_uninitialized.rs` | Storage without an initialized value |
| `examples/03_out_of_bounds.rs` | One-past pointer versus readable element |
| `examples/04_data_race.rs` | Two unsynchronized non-atomic writes |
| `examples/05_lost_update.rs` | Atomic accesses, incorrect combined operation |
| `examples/06_publication.rs` | Optional: flag does not publish payload yet |
| `src/lib.rs` | Four participant repair puzzles A–D |
| `tests/baseline.rs` | Two working tests to confirm setup |
| `tests/challenge.rs` | Four acceptance tests for A–D |
| `WORKSHEET.md` | Evidence record and discussion questions |
| `HINTS.md` | Staged hints; open after an attempt |

## 0. Setup and smoke test (first 10 minutes)

Open the participant repository in VS Code. In its terminal, enter this directory:

```sh
cd miri-lab
```

All commands below run **inside `miri-lab`**. If unsure, run `pwd`; its final
component should be `miri-lab`. The pinned toolchain matches the main labs.
Only if this nightly or its components are missing, run the installation commands
while online, before class:

```sh
rustup toolchain install nightly-2026-07-22 --profile minimal --component miri --component rust-src
cargo +nightly-2026-07-22 miri setup
```

Now compile all targets without executing the unfinished puzzles:

```sh
cargo +nightly-2026-07-22 check --all-targets --offline
```

Unused-variable warnings in the starter are expected. Do not run `cargo fix`:
the variables are there for your implementations. Check only checks compilation;
it does not execute the pointer operations or prove their safety.

Run just the two supplied baseline tests, first natively and then through Miri:

```sh
cargo +nightly-2026-07-22 test --test baseline --offline
cargo +nightly-2026-07-22 miri test --test baseline --offline
```

Both should report **2 passed**. Do not run all challenge tests yet: their
`todo!()` bodies intentionally panic until implemented. `--offline` prevents
Cargo dependency fetching; it is not a Miri safety setting.

## 1. Learn to read the report (10 minutes)

For each experiment: predict → run → identify the first invalid operation →
explain the violated rule → repair → save → rerun the same command.

A report has three useful parts:

1. **Category:** `Undefined Behavior`, a compiler error, a normal panic, or
   `unsupported operation`. These mean different things.
2. **Location:** filename and source line of the access that failed.
3. **History:** the relevant allocation/free or conflicting access, when available.

Copy those into `WORKSHEET.md`. Do not start with the final generic Cargo
`process didn't exit successfully` line. Miri stops on the first detected error;
run each example separately. Diagnostic wording and allocation IDs may vary.

## 2. Three memory investigations (20 minutes)

The invalid-memory examples have a guard against accidental native execution.
Use the following Miri commands; each example is **expected to fail before repair**.
Do not disable Miri checks to obtain a passing result.

### Experiment 1: an address survives its owner

Open `examples/01_dangling.rs`. Follow the owner from allocation to `drop`, then
follow the raw pointer to its read. Does storing an address keep a Box alive?

```sh
cargo +nightly-2026-07-22 miri run --example 01_dangling --offline
```

Expected category: UB involving a freed allocation. Repair the order of actions
so the owner lives through the last read. Preserve the read and the cleanup;
do not leak the allocation or remove the experiment. Save and rerun.
Connection: a pointer cached before MiniVec growth or after a database close.

### Experiment 2: parking spaces are not parked values

Open `examples/02_uninitialized.rs`. `MaybeUninit` reserves storage without
promising that a valid value is in it. `assume_init` makes that promise yourself.

```sh
cargo +nightly-2026-07-22 miri run --example 02_uninitialized --offline
```

Expected category: UB involving uninitialized data. Write 42 into the slot
before assuming it is initialized, keep `MaybeUninit`, add a safety explanation,
save and rerun. Connection: MiniVec capacity is not its initialized length.

### Experiment 3: the end marker is not an element

Open `examples/03_out_of_bounds.rs`. There are three u32 values, each four bytes.
`pointer.add(3)` is the one-past address; an attempted read needs another four bytes.

```sh
cargo +nightly-2026-07-22 miri run --example 03_out_of_bounds --offline
```

Expected category: UB involving an access at/beyond the allocation's end.
Keep the raw-pointer experiment but read the last valid element of this known
nonempty array. Explain why that expression would need an empty-input check in
a reusable function. Save and rerun. Connection: `index < len`, not `index <= len`.

## 3. Transfer puzzles A and B (10 minutes, then break)

Edit only the marked bodies in `src/lib.rs`:

- A: implement `checked_read`, including empty input and out-of-range indices,
  using safe Rust. Return the copied value as `Option<u32>`.
- B: implement `initialize_slot`. Keep the supplied slot, write the input value
  before reading, and justify the unsafe operation with a safety comment.

Save first. Compile, then run the focused test for the body you completed:

```sh
cargo +nightly-2026-07-22 check --offline
cargo +nightly-2026-07-22 test --test challenge a_read_boundaries --offline
cargo +nightly-2026-07-22 miri test --test challenge a_read_boundaries --offline
```

After B is implemented:

```sh
cargo +nightly-2026-07-22 test --test challenge b_initialize_before_reading --offline
cargo +nightly-2026-07-22 miri test --test challenge b_initialize_before_reading --offline
```

Each filtered run should report **1 passed**, not zero tests. A passing Miri run
with zero selected tests gives no evidence about the function you meant to test.

## 4. Concurrency: a real data race (20 minutes)

Open `examples/04_data_race.rs`. Predict what `unsafe impl Sync` promises.
`UnsafeCell` allows interior mutation; it provides no locking. Two threads
perform conflicting writes with no synchronization ordering them.

```sh
cargo +nightly-2026-07-22 miri run --example 04_data_race --offline
```

Expected category: UB/data race, with the two conflicting accesses reported.
A CPU running one instruction at a time does not establish the required
happens-before relationship. Joining both threads at the end does not order
their earlier writes relative to each other.

Repair this example with a `Mutex<u32>`, replacing the custom Shared wrapper and
its manual Sync implementation. Each write must hold a lock guard. Allow either
1 or 2 as the final result: mutual exclusion does not choose which writer wins.
Save and rerun. Do not use sleeps as synchronization.

Then implement puzzle C, `increment_locked`, keeping its read and write under
**one guard**. After saving:

```sh
cargo +nightly-2026-07-22 test --test challenge c_locked_counter_keeps_all_updates --offline
cargo +nightly-2026-07-22 miri test --test challenge c_locked_counter_keeps_all_updates --offline
```

The test runs four threads with sixteen increments each: expected count 64.
Infrastructure example: protecting shared cache metadata during concurrent updates.

## 5. Concurrency: all accesses atomic, answer still wrong (20 minutes)

Open `examples/05_lost_update.rs`. The barrier makes both threads read the old
count before either writes. Draw this sequence: read 0, read 0, store 1, store 1.
This example uses safe Rust and may be run natively as well as under Miri:

```sh
cargo +nightly-2026-07-22 run --example 05_lost_update --offline
cargo +nightly-2026-07-22 miri run --example 05_lost_update --offline
```

Expected: an ordinary assertion panic: observed 1, expected 2. This is a logical
race, not a non-atomic data race. Miri runs assertions too; it does not invent the
business requirement that two increments must produce two.

Repair the increment as one atomic read-modify-write operation. Keep two workers,
the barrier and the final assertion. Stronger ordering alone cannot combine a
separate load and store; the broken version already uses SeqCst.

Then implement puzzle D, `increment_atomic`. Explain why a counter that publishes
no other memory can use Relaxed for its increment. Save, then:

```sh
cargo +nightly-2026-07-22 test --test challenge d_atomic_counter_keeps_all_updates --offline
cargo +nightly-2026-07-22 miri test --test challenge d_atomic_counter_keeps_all_updates --offline
```

Infrastructure example: independent request counters and metrics. Do not generalize
this ordering argument to a flag protecting a separate payload.

## 6. Debrief before break (10 minutes)

Complete the report for all five experiments. Explain why an ordinary successful
native run of unsafe code would not establish safety. Discuss the difference
between protecting memory and implementing the intended algorithm.

## 7. Validate and investigate more schedules (20 minutes)

Only after A–D are implemented, run the whole library test suite:

```sh
cargo +nightly-2026-07-22 test --offline
cargo +nightly-2026-07-22 miri test --offline
```

Expected: **2 baseline + 4 challenge tests pass**; zero library tests and zero
doctests are normal for this crate. Cargo test does not execute the main functions
of the demonstration examples. Run each repaired example separately as above.

Try more scheduling choices for the two counter tests:

```sh
for seed in 0 1 2 3 4; do
  MIRIFLAGS="-Zmiri-seed=$seed" cargo +nightly-2026-07-22 miri test --test challenge counter --offline || break
done
```

Each run should select **2 tests**. `MIRIFLAGS` applies to that command; a seed
selects different interpreter choices, including scheduling. Record a failing seed
and rerun with it when investigating. Five seeds do not explore all possible executions.

## 8. Evidence, FFI and engineering decisions (30 minutes)

Discuss and answer in the worksheet:

- Does a clean Miri run prove that a safe API is sound for every possible caller?
- What would a test that never triggers MiniVec growth fail to cover?
- If Miri reports an unsupported foreign call, has it found UB?
- In the earlier FFI lab, the Miri path uses a Rust mock. Which native-C behavior
  still needs testing against the real C implementation?
- Which production tests would you prioritize given Miri's execution cost?

Deliver: A–D implementations, five repaired experiments, the completed worksheet,
and native/Miri test output. Include the toolchain and seeds you used. Add a
safety argument for each remaining unsafe block; passing tests supplement it.

## Optional advanced extension: publishing a separate payload (25–35 minutes)

Open `examples/06_publication.rs`. One writer stores 42 in non-atomic memory,
then sets an atomic ready flag. One reader waits for the flag, then reads memory.
There is exactly one publication, no reset, no reuse and no second writer.

```sh
cargo +nightly-2026-07-22 miri run --example 06_publication --offline
```

Expected on the pinned toolchain: UB/data race. The atomic flag itself is accessed
atomically; the separate payload is missing synchronization.

Change only the store/load memory orderings and update the safety explanation.
Preserve the non-atomic payload and single writer/reader. Identify the smallest
suitable pair of orderings, explain which observed value connects them, then
save and rerun. Also explain why the counter's Relaxed choice was valid while
this flag's Relaxed choice is insufficient. Try seeds 0–4 using the loop above,
replacing `miri test --test challenge counter` with `miri run --example 06_publication`.

Do not turn this one-shot demonstration into a reusable concurrent mailbox by
resetting the flag: that would need a new protocol and safety argument.
For production, consider a channel or established synchronization type first.

## Limits and primary references

Miri checks the executions you exercise. It is not a proof of soundness, does
not explore all interleavings, and has incomplete weak-memory exploration.
Most arbitrary foreign calls are unsupported. A Rust mock exercises the Rust
path and the mock's assumptions, not the actual native C implementation.
For systematic bounded concurrency exploration, investigate Loom separately.

- [Miri: usage, detected errors and limitations](https://github.com/rust-lang/miri)
- [Rust's undefined behavior rules](https://doc.rust-lang.org/reference/behavior-considered-undefined.html)
- [UnsafeCell and concurrency requirements](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html)
- [Atomic orderings](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)
- [Loom](https://docs.rs/loom/latest/loom/)

## Advanced capstone

After the guided lab, try the [snapshot handoff puzzle](../miri-puzzle/README.md).
It combines these concepts in one ownership and concurrency protocol, with eight
implementation tasks and a 90-minute timer.
