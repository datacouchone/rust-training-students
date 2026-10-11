# Day 1: memory, unsafe abstractions and FFI

Open Participants.code-workspace in the standalone participant repository
(or Students.code-workspace in the original student edition). Commands below
run from this repository's root. Attendees should already know Rust syntax,
ownership, borrowing, traits, lifetimes and basic error handling.

## Preflight before class

```sh
rustup toolchain install nightly-2026-07-22 --component miri
cargo +nightly-2026-07-22 fetch --manifest-path labs/Cargo.toml
cargo +nightly-2026-07-22 check --manifest-path labs/Cargo.toml -p lab1-layout-starter -p lab2-minivec-starter -p lab3-ffi-starter
cargo +nightly-2026-07-22 miri setup
cargo +stable check --all-targets --manifest-path exercises/day-01/Cargo.toml --offline
```

Lab 3 also needs a C compiler (on macOS, Command Line Tools). Fetch dependencies
before class. Miri startup should happen before timed work. TODO bodies compile
but tests executing them fail until implemented. Layout's TODO tests are ignored
initially, so its default green result does not mean you completed the exercise.

## Hourly rhythm

| Session | Core work | Result |
|---|---|---|
| 09:00–09:50 | 20-minute introduction, then representation and padding predictions | Distinguish documented layout from a measurement |
| 10:00–10:50 | Lab 1 reorder/boxing work and debrief | Explain layout plus heap-allocation trade-offs |
| 11:00–11:50 | MiniVec invariant, empty state and allocation | State which slots contain live T values |
| 12:00–13:00 | Lunch | |
| 13:00–13:50 | Growth, push/pop and bounds | Preserve ownership and initialized range |
| 14:00–14:50 | Drop, slice construction, safety review and Miri | Defend each unsafe operation |
| 15:00–15:50 | C ownership contract, validation and wrapper work | Distinguish borrowed pointers from owners |
| 16:00–16:50 | Finish FFI, verify and transfer the reasoning | Explain safe public API and validation limits |

Take a ten-minute break after every teaching session. Each lab spans the
explanation and practice sessions shown here, rather than only a TODO sprint.

## Lab commands

```sh
cargo +nightly-2026-07-22 test --manifest-path labs/Cargo.toml -p lab1-layout-starter -- --include-ignored
cargo +nightly-2026-07-22 test --manifest-path labs/Cargo.toml -p lab2-minivec-starter
cargo +nightly-2026-07-22 miri test --manifest-path labs/Cargo.toml -p lab2-minivec-starter
cargo +nightly-2026-07-22 test --manifest-path labs/Cargo.toml -p lab3-ffi-starter
cargo +nightly-2026-07-22 miri test --manifest-path labs/Cargo.toml -p lab3-ffi-starter
```

Keep the tests and method contracts. The instructor demonstrates selected pieces,
then you implement other pieces and explain a changed case. Use
exercises/day-01/README.md for independent extensions when ready. You do not need
to complete every extension. A clean Miri run is evidence about tested Rust
executions. Lab 3 uses a Rust mock under Miri and native C during ordinary tests.

## Keeping your work and comparing answers

Work on a personal branch, for example `git switch -c my-day1`. Commit your own
attempt before updating from the instructor repository. Do not overwrite your
files with reference code while trying to understand a failure.

After the instructor announces a release, `git fetch origin --tags` obtains the
`day-01-solutions` tag. Inspect the separate `answers/day-01/` directory at that
revision, or merge the instructor's main branch after committing your work.
The original starters stay available for fresh attempts. No solution release is
present in the initial participant repository.
