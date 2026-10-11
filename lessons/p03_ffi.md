# P03 — Pointer and length at a foreign boundary

Concept: An ABI signature does not prove pointer validity; a safe wrapper must establish the callee contract.

Open `../concepts/src/p03_ffi.rs`. Work only in the marked puzzle bodies.
The surrounding API and tests are scaffolding; do not change their contracts.

## Together: instructor-led puzzle (5–10 minutes)

Wrap foreign_sum with a safe slice API and a justified unsafe call.

Before the instructor types the fix, predict what each relevant test requires.
Run from `students/`:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p03_ffi::tests::demo --offline
```

## Your turn: transfer puzzle (8–15 minutes)

Wrap foreign_max with the same pointer/length contract, including empty input.

Write your own implementation, predict a boundary case, then run:

```sh
cargo +stable test --manifest-path concepts/Cargo.toml p03_ffi::tests::exercise --offline
```

Done: the exercise tests pass, you explain the concept in your own words,
and you justify one edge case. A passing test alone is not the whole explanation.
The unfilled puzzle bodies compile but panic when executed; failures are
expected until each puzzle is solved. Run only the current puzzle's tests.

## Further practice

Continue with `../labs/lab3-ffi/starter/` for the advanced application.
Read that lab's puzzle card before running its commands. For P00, use
`../labs/prereq-check/` instead. This mini-puzzle introduces one part of the
module; the advanced lab covers the wider concepts listed in the course map.

Primary reference: [Pointer and length at a foreign boundary](https://doc.rust-lang.org/nomicon/ffi.html).
