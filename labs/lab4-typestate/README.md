# lab4-typestate — advanced concept puzzles

Concepts: state transitions; const generics; lending iteration and GATs.

## Together, then your turn
Solve bind together; students solve connect/send, then lending next.

Open `starter/src/lib.rs` (lab9 also has `server.rs`, `state.rs`, and `frame.rs`).
Read contracts and tests, then work on one TODO at a time. Existing scaffolding
is deliberate: only the concept under discussion should need completion.
For lab9, the puzzle is investigation and repair of faults rather than filling
every working server function. Record reproduction, evidence and root cause.

Run from `students/labs/`:

```sh
cargo test -p lab4-typestate-starter
```

Done: state transitions and mutable windows; explicitly test forbidden-state calls too. Use the pinned nightly. Unfinished TODOs can cause test
panics; they are expected. This card is a puzzle sequence, not permission to
skip the safety reasoning or operational prerequisites.

For lab9's core exercise, build the Docker image from this labs directory:
`docker build -t rust-student-core -f lab9-core-dump/docker/Dockerfile .`
Its Docker assets target the student package. Docker and Linux debugger
behavior need a separate rehearsal. The image uses the lockfile in this kit.
