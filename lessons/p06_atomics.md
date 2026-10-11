# P06 — Atomic counters and publication

Concept: Atomicity and synchronization are different; counters and publication need different reasoning.

Open `../concepts/src/p06_atomics.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Return the previous value while incrementing an atomic ID counter; justify the ordering.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p06_atomics::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Use a ready flag to publish and observe prior writes; name the required ordering pair.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p06_atomics::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/lab6-atomics/starter/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [Atomic counters and publication](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html).
