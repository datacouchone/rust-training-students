# P05 — Macros and expression evaluation

Concept: Macro expansion duplicates syntax; bind an expression once when it must execute once.

Open `../concepts/src/p05_macros.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Expand make_pair! for a Copy expression while evaluating that expression only once.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p05_macros::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Expand repeat_vec! for Clone values; evaluate the value expression only once.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p05_macros::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/lab5-derive-macro/starter/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [Macros and expression evaluation](https://doc.rust-lang.org/reference/macros-by-example.html).
