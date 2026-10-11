# P08 — Chunk processing and correctness before timing

Concept: A chunked algorithm must handle the remainder and be equivalent before comparing speed.

Open `../concepts/src/p08_performance.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Sum u32 values into u64 in chunks of four, including all tail elements.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p08_performance::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Count values at or above a threshold in one pass without allocating an intermediate Vec.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p08_performance::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/lab8-simd-perf/starter/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [Chunk processing and correctness before timing](https://doc.rust-lang.org/std/slice/struct.ChunksExact.html).
