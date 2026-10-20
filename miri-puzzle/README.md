# Miri capstone — the snapshot handoff

**90–120 minutes.** One connected puzzle: build a batch of owned values, hand it
to another thread once, and account for the work. Intended after the Miri lab.
The APIs compile, but eight marked bodies need implementation.

A storage worker fills part of a fixed-capacity batch. Multiple workers may
compete to publish a batch, but only one batch is accepted. Multiple consumers
may compete to take it, but exactly one may receive its ownership. Rejected
publishers keep their original value. Unclaimed payloads must be cleaned up.

This is an in-memory teaching model, not a production queue or a reusable channel.

## Open these files

- `src/lib.rs`: all eight TODOs and the supplied type/constructor scaffolding.
- `tests/challenge.rs`: fourteen behavioral and concurrent acceptance tests.
- `tests/baseline.rs`: two working setup tests.
- `TRAPS.md`: questions to defend your design.
- `HINTS.md`: staged hints, to open after an attempt.
- `timer.html`: independent 90-minute timer with pause/resume.

## The rules you must preserve

### 1. Batch<T, N>: storage, initialized values and ownership

`slots` owns room for N values. Only the prefix `[0, len)` contains live T values.
The rest is uninitialized storage, even if it is inside the allocation.

Implement A–D:

- **A / push:** when full return `Err(value)` with that exact original owner.
  Otherwise initialize one spare slot before recording it as live.
- **B / pop:** transfer the last value out, excluding it from future borrowing
  and destruction. Empty returns None. Reusing a vacated slot must work.
- **C / get:** return a reference tied to this batch only for initialized indices.
  Reject empty, one-past and `usize::MAX` indices before pointer arithmetic.
- **D / Drop:** destroy each remaining live value once, then allow the allocation
  to be released. Never destroy unused slots or values already transferred out.

Support N = 0, zero-sized T, partially filled batches, String and other non-Copy
values. Keep the Box/MaybeUninit representation. Do not add T: Copy/Clone or leak
values to avoid a destructor. Do not manufacture a longer reference lifetime.
A safe implementation is welcome where it satisfies the existing representation
and contract; unsafe blocks must each have a specific safety argument.

### 2. OneShot<T>: one publication and one transfer

The only lifecycle is:

```text
EMPTY -> WRITING -> READY -> TAKEN
```

Implement E–G:

- **E / try_publish:** at most one publisher reserves initialization. It must
  claim before touching the slot, publish after initialization, and return the
  original T on failure. Never invoke a user callback while reserving the slot.
- **F / try_take:** a failed attempt must not close an empty or writing handoff.
  Only one successful taker may transfer T out. Observing the publication must
  establish the required synchronization with the payload initialization.
- **G / Drop:** destroy an accepted but unclaimed payload. Do not destroy an
  empty slot or a payload whose ownership has moved to a consumer.

Keep the UnsafeCell/MaybeUninit/atomic-state representation and the supplied
public APIs. Do not replace the handoff with a Mutex, channel, OnceLock or leaked
allocation: the exercise is to justify this specific protocol. Keep all four
states. Do not reset or reuse the handoff. Do not hide state using cfg(miri),
disable Miri checks, add sleeps, or weaken tests to get a passing result.

Explain every chosen memory ordering and audit `unsafe impl<T: Send> Sync`.
Why is Send required for the payload? Must the payload also be Sync if ownership
moves and no shared payload reference escapes?

`try_take` is nonblocking. None means this attempt did not transfer a value;
it does not mean another thread can never publish later. The tests use a bounded
yield loop to avoid an infinite wait when an implementation is incorrect.

### 3. Audit: atomic ticket allocation and a coherent journal

Implement H / record:

- Allocate one ticket ID per call as one atomic update.
- Update completion count, byte sum and entries under the journal's mutex.
- A concurrent snapshot must see count == entry count and the matching byte sum.

Tickets need not appear in completion order. The ticket allocator does not
publish the contents of the journal. Snapshot covers completed records; another
thread may have allocated a ticket but not yet finished recording it.
Workloads in this exercise do not overflow usize, locks are not poisoned, and
payload destructors do not panic. Justify the scope of the lock guard.

## Run commands from the participant repository root

Start by compiling, then run only the two supplied baseline tests:

```sh
cargo +nightly-2026-07-22 check --manifest-path miri-puzzle/Cargo.toml --all-targets --offline
cargo +nightly-2026-07-22 test --manifest-path miri-puzzle/Cargo.toml --test baseline --offline
cargo +nightly-2026-07-22 miri test --manifest-path miri-puzzle/Cargo.toml --test baseline --offline
```

Expect **2 baseline tests passed**. Unused-variable warnings are normal before
implementation. Keep the APIs and TODO variables; do not apply cargo fix.

### Stage A–D: batch only (about 25–30 minutes)

Save all four Batch bodies before running their tests. Drop must be complete
before testing nonempty batches.

```sh
cargo +nightly-2026-07-22 test --manifest-path miri-puzzle/Cargo.toml --test challenge batch_ --offline
cargo +nightly-2026-07-22 miri test --manifest-path miri-puzzle/Cargo.toml --test challenge batch_ --offline
```

Expect six selected batch tests. Read the selected names and count before claiming a pass.

### Stage E–G: publication and cleanup (about 30–35 minutes)

Complete all three handoff bodies first; use a plain String payload before
combining it with Batch.

```sh
cargo +nightly-2026-07-22 test --manifest-path miri-puzzle/Cargo.toml --test challenge oneshot_ --offline
cargo +nightly-2026-07-22 miri test --manifest-path miri-puzzle/Cargo.toml --test challenge oneshot_ --offline
```

Expect four selected tests. Then run the critical publication test:

```sh
cargo +nightly-2026-07-22 miri test --manifest-path miri-puzzle/Cargo.toml --test challenge publication_before_join --offline
cargo +nightly-2026-07-22 miri test --manifest-path miri-puzzle/Cargo.toml --test challenge send_but_not_sync --offline
```

The first test reads before joining the producer. Moving the read after the
join could hide missing publication synchronization. Do not change that test.

### Stage H and full integration (about 20–25 minutes)

After record and all previous methods are saved:

```sh
cargo +nightly-2026-07-22 test --manifest-path miri-puzzle/Cargo.toml --test challenge audit_ --offline
cargo +nightly-2026-07-22 miri test --manifest-path miri-puzzle/Cargo.toml --test challenge audit_ --offline
cargo +nightly-2026-07-22 test --manifest-path miri-puzzle/Cargo.toml --offline
cargo +nightly-2026-07-22 miri test --manifest-path miri-puzzle/Cargo.toml --offline
```

Complete result: **2 baseline + 14 challenge + 3 compile-fail doctests**.
The doctests can reject misuse before your bodies are finished; they test the
API restrictions, not implementation completion. Zero selected tests prove
nothing about the code you meant to exercise.

### Try more interpreter choices

```sh
for seed in 0 1 2 3 4; do
  MIRIFLAGS="-Zmiri-seed=$seed" cargo +nightly-2026-07-22 miri test --manifest-path miri-puzzle/Cargo.toml --test challenge --offline || break
done
```

Each seed selects all fourteen challenge tests. Record a failing seed to
reproduce it. Passing these runs does not prove soundness or cover every schedule.

## Deliverables and debrief (15–20 minutes)

1. Eight implementations and a safety comment at every unsafe operation.
2. Native and Miri results, with toolchain and scheduling seeds recorded.
3. Your state/ownership table: for each state, who may access the slot, whether
   T is initialized, and who will destroy it.
4. Answers to TRAPS.md. Identify one invariant tested by an assertion and one
   invalid memory access that Miri would detect.

This puzzle is about both memory validity and correct behavior. Passing Miri
without the expected accounting/ownership outcomes is insufficient. Native tests
alone are also insufficient evidence for the unsafe protocol.

## Primary references

- [Miri: usage and limits](https://github.com/rust-lang/miri)
- [MaybeUninit](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html)
- [UnsafeCell](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html)
- [Atomic orderings](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)
