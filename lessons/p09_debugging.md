# P09 — Parse errors and framed input

Concept: Malformed input should yield an explicit error rather than an uncontrolled panic.

Open `../concepts/src/p09_debugging.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Parse name=value into a Metric; reject missing names, separators, and non-finite values.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p09_debugging::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Decode one complete big-endian u32-length-prefixed UTF-8 frame; reject malformed frames.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p09_debugging::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/lab9-core-dump/starter/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [Parse errors and framed input](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html).
