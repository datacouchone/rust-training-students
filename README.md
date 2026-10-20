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


## Fresh-start policy

This repository contains unfinished demo and transfer puzzles and the original
exercise verification tests. It contains no private trainer speaking notes or
recording workspaces. Reference solutions, when released, are separate from the
unfinished starter files; the Day 2 Miri exercises remain unsolved.
Some supporting code is intentionally supplied (such as the FFI backend and
scalar performance baseline). Implement the marked puzzle bodies yourself.

Open `Participants.code-workspace`. Start with `DAY-01.md`; keep your changes
on a personal branch. Public slides are teaching material and may include short
worked examples; they are separate from the unfinished assignment files.


## Day 2: Miri

Start with the [Miri lab](miri-lab/README.md): six memory and concurrency
investigations, four repair exercises, baseline tests and a worksheet.
Then try the optional [snapshot handoff capstone](miri-puzzle/README.md):
eight implementation tasks combining ownership, initialization, atomic
publication and concurrent accounting.

Both directories contain unfinished participant exercises. Run their baseline
tests first; challenge tests become ready as you complete the marked bodies.
Follow each README for the pinned toolchain, setup and exact commands.


## Lab 4 and Lab 5 advanced challenges

- [Transactional journal](lab4-puzzle/README.md): typestate, cancellation,
  bounded ownership and lending across physical wrap; six implementation tasks.
- [Policy-aware derive](lab5-puzzle/README.md): generic bounds, renaming,
  redaction, hygiene and source-aware diagnostics; four implementation sections.

Each challenge has setup tests, staged acceptance commands, hints, defense
questions, rejected-caller fixtures and its own timer. Start with the baseline
commands in each README. The participant implementation sections remain unfinished.
