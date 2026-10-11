# P04 — State transitions encoded in types

Concept: Put operations on the valid state type and consume self when transitioning.

Open `../concepts/src/p04_typestate.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Implement Closed -> Open without losing the door's name.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p04_typestate::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Implement Open -> Closed; verify that walking through a closed door cannot compile.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p04_typestate::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/lab4-typestate/starter/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [State transitions encoded in types](https://doc.rust-lang.org/book/ch10-01-syntax.html).
