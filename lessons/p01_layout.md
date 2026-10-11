# P01 — Alignment and padding

Concept: A field starts at an offset divisible by its alignment; structs need tail padding too.

Open `../concepts/src/p01_layout.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Compute padding before a field for a nonzero power-of-two alignment.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p01_layout::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Compute the final padded size for a nonzero power-of-two struct alignment.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p01_layout::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/lab1-layout/starter/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [Alignment and padding](https://doc.rust-lang.org/reference/type-layout.html).
