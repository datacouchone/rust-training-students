# P00 — Values, functions and borrowing

Concept: Borrow text to inspect it while the caller retains ownership.

Open `../concepts/src/p00_basics.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Return the UTF-8 byte length of borrowed text.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p00_basics::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Return a new owned greeting without taking ownership of the input text.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p00_basics::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/prereq-check/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [Values, functions and borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html).
