# P02 — Safe interfaces over unsafe operations

Concept: Every unsafe operation needs a specific validity argument; bounds checks can establish it.

Open `../concepts/src/p02_unsafe.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Return a checked reference using pointer arithmetic; return None outside bounds.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p02_unsafe::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Swap two elements through pointers only after checking both indices; otherwise return false.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p02_unsafe::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/lab2-minivec/starter/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [Safe interfaces over unsafe operations](https://doc.rust-lang.org/std/ptr/fn.swap.html).
