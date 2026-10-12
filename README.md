# Rust concept puzzles — student edition

Open `Participants.code-workspace` in VS Code. Start with `COURSE-MAP.md` and the
current lesson card. Each lesson has an instructor-led demo puzzle followed
by a separate transfer puzzle for you to solve. Each body marked `demo` or
`exercise` is intentionally unfinished. Keep the API contracts and tests.

Run from this directory:

```sh
cargo +stable check --all-targets --manifest-path concepts/Cargo.toml --offline
cargo +stable test --manifest-path concepts/Cargo.toml p01_layout::tests::demo --offline
cargo +stable test --manifest-path concepts/Cargo.toml p01_layout::tests::exercise --offline
```

Change the filter to your current concept. Initial checks should compile;
tests that execute unfinished bodies should fail. Terminal > Run Task offers
the same filtered choices. P04 includes an invalid-state compile-fail doctest.
P10 is a discussion case, with no new Rust implementation required.

The `concepts/` crate uses stable Rust and has no external dependencies.
The `labs/` workspace contains the advanced applications and uses its pinned
nightly, external crate dependencies, C tooling for FFI, and Docker for the
core-dump walkthrough. Read each puzzle card before running the lab tests.
The complete course assumes the prerequisites in `labs/prereq-check/`.

Try first, predict an edge case, then request a hint. Completion means you
can explain why the code works as well as passing its tests.

## Entry point for this advanced course

Attendees already know Rust: begin delivery with P01 / Module 1, using a
struct-layout prediction. P00 is optional prerequisite practice. Keep explanations short, ask for a prediction, then
run a demo and use the lab to practise reasoning. Fast finishers use the
lab's stretch goals; allow others to finish the core contract and debrief.

## Day 1

Start with [DAY-01.md](DAY-01.md). Open presentations/day-01/index.html for the
public interactive deck. Extra practice is in exercises/day-01/. Your original
puzzles remain available after reference answers are published separately.

