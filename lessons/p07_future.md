# P07 — Future polling and wakeups

Concept: Pending needs a path to a wakeup; polling is not the same as blocking a thread.

Open `../concepts/src/p07_future.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Make Countdown return Pending with a wakeup while remaining > 0, then Ready.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p07_future::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Make AddLater return Pending once with a wakeup, then Ready with its sum.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p07_future::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/lab7-executor/starter/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [Future polling and wakeups](https://doc.rust-lang.org/std/future/trait.Future.html).
