# Lab 4 challenge: transactional bounded journal

**90–120 minutes.** Combine typestate, const generics, ownership and GAT lending
in one connected puzzle. Six marked bodies, A–F, need implementation.

A storage worker stages events without changing committed history. Preparation
changes the transaction's type. Commit publishes the events and returns evicted
owners. Cancelling either transaction state leaves committed history unchanged.
Later, edit overlapping logical windows, including windows crossing physical wrap.

This is a single-threaded, in-memory model. It provides neither persistence nor
database crash recovery. Staging uses a Vec; committed slots are inline. Workloads
do not exhaust memory and element destructors do not panic.

## Start here

- src/lib.rs: supplied types and A–F.
- tests/baseline.rs: two working setup tests.
- tests/challenge.rs: fourteen acceptance tests.
- ui/: four callers that must fail compilation.
- check_rejections.py: verifies the intended compiler errors.
- HINTS.md: staged hints after an attempt.
- TRAPS.md: questions to defend.
- timer.html: 90-minute timer, independently adjustable.

Use the participant repository root for every command below.

## The ownership rules

- Open can stage at most N values. A refusal returns that exact owner.
- Empty prepare returns the original Open transaction, still usable.
- Successful prepare consumes Open. Only Prepared offers commit.
- Dropping Open or Prepared drops pending owners without publishing them.
- Commit applies pending values in order and returns evictions oldest-first.
- Do not clone values, add Copy/Clone bounds, publish during stage/prepare,
  use unsafe code, change the public API, or linearize wrapped storage.
- N = 0 is supported: stage refuses every value; windows yield nothing.
- Unlike the core ring, head identifies the oldest committed slot.

## 1. Compile and smoke test, before implementing anything

    cargo +nightly-2026-07-22 check --manifest-path lab4-puzzle/Cargo.toml --all-targets --offline
    cargo +nightly-2026-07-22 test --manifest-path lab4-puzzle/Cargo.toml --test baseline --offline
    cargo +nightly-2026-07-22 miri test --manifest-path lab4-puzzle/Cargo.toml --test baseline --offline

Expect two baseline tests passed. Unused starter fields are expected. Check does
not execute TODOs. Run the acceptance stages only after their bodies are saved.

## 2. A–B: stage and prepare, about 20 minutes

Implement stage and prepare. Preparation validates nonempty staging and changes
type; it does not modify committed storage. Let ordinary field destruction
cancel pending ownership.

    cargo +nightly-2026-07-22 test --manifest-path lab4-puzzle/Cargo.toml --test challenge stage_ --offline
    cargo +nightly-2026-07-22 miri test --manifest-path lab4-puzzle/Cargo.toml --test challenge stage_ --offline

Expect five selected tests passed. C/D are not required for these tests.

## 3. C–D: publish and evict, about 30 minutes

Implement commit and the private push_committed. Reserve the eviction vector
before modifying the journal. Retain the latest N values and preserve ownership
of anything evicted. Try a capacity-three journal already holding 1,2,3, then
commit 4,5: return 1,2 and retain 3,4,5.

    cargo +nightly-2026-07-22 test --manifest-path lab4-puzzle/Cargo.toml --test challenge journal_ --offline
    cargo +nightly-2026-07-22 miri test --manifest-path lab4-puzzle/Cargo.toml --test challenge journal_ --offline

Expect five selected tests passed.

## 4. E–F: logical lending windows, about 25 minutes

Implement next and Window::get_mut. A returned window borrows the journal for
one next call. A local element borrow lasts for its get_mut call's borrow.
Zero width, incomplete windows and overflowing end positions yield None.
Do not assume logical neighbours are physically adjacent.

    cargo +nightly-2026-07-22 test --manifest-path lab4-puzzle/Cargo.toml --test challenge windows_ --offline
    cargo +nightly-2026-07-22 miri test --manifest-path lab4-puzzle/Cargo.toml --test challenge windows_ --offline

Expect four selected tests passed.

## 5. Final acceptance and explanation

    cargo +nightly-2026-07-22 test --manifest-path lab4-puzzle/Cargo.toml --offline
    cargo +nightly-2026-07-22 miri test --manifest-path lab4-puzzle/Cargo.toml --offline
    python3 lab4-puzzle/check_rejections.py

Expect two baseline plus fourteen challenge tests passed, followed by four
verified compiler rejections. Zero library tests/doctests are normal here.
The rejection runner expects compilation failure, not a running panic.

Explain one cancellation trace, one wrapped commit trace and one overlapping
window trace. A green run is evidence for those tests, not a correctness proof.
