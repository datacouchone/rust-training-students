# Lab 4 advanced challenge: reference solution

[solution.rs](solution.rs) contains the complete implementation of A–F.
The starter in `src/lib.rs` remains unfinished so you can keep practising.

Read or compare the reference first. To run it against the supplied tests,
save your own attempt somewhere safe, then run from the repository root:

```sh
cp lab4-puzzle/solution.rs lab4-puzzle/src/lib.rs
cargo +nightly-2026-07-22 test --manifest-path lab4-puzzle/Cargo.toml --offline
cargo +nightly-2026-07-22 miri test --manifest-path lab4-puzzle/Cargo.toml --offline
python3 lab4-puzzle/check_rejections.py
```

The copy command replaces your local implementation. It does not change the
published starter. Expect two baseline and fourteen challenge tests in both
native and Miri runs, plus four intended compiler rejections.

## Implementation guide

- **A — stage:** refuse a full pending batch by returning the original owner;
  otherwise stage the value without changing the journal.
- **B — prepare:** return the original empty transaction, or consume Open and
  carry its journal borrow and pending values into Prepared.
- **C — commit:** reserve eviction output space first, publish in staging
  order, and transfer evicted values to the caller.
- **D — push_committed:** append into spare slots or replace the oldest slot
  and advance head; retained values stay in logical order across wrap.
- **E — next:** check width and end bounds, advance one logical position,
  and lend a window tied to the current mutable receiver borrow.
- **F — get_mut:** validate the window index before arithmetic, map the
  logical position through head, and borrow the stored value.

Dropping either transaction state cancels pending work through normal field
destruction. The reference keeps `forbid(unsafe_code)` and adds no Copy or
Clone restrictions on stored values. This remains an in-memory teaching model,
not a persistent or concurrent database journal. Passing tests and Miri runs
are evidence for exercised cases, not a proof for every possible input.
