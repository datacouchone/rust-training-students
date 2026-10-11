# lab3-ffi — advanced concept puzzles

Concepts: null handling; C strings; pointer ownership; RAII; Rust exported C ABI.

## Together, then your turn
Walk through the open-handle contract; students fill get/close/checksum duties.

Open `starter/src/lib.rs` (lab9 also has `server.rs`, `state.rs`, and `frame.rs`).
Read contracts and tests, then work on one TODO at a time. Existing scaffolding
is deliberate: only the concept under discussion should need completion.
For lab9, the puzzle is investigation and repair of faults rather than filling
every working server function. Record reproduction, evidence and root cause.

Run from `students/labs/`:

```sh
cargo test -p lab3-ffi-starter
```

Done: valid/invalid input and one-close ownership; then the README Miri limitations. Use the pinned nightly. Unfinished TODOs can cause test
panics; they are expected. This card is a puzzle sequence, not permission to
skip the safety reasoning or operational prerequisites.

For lab9's core exercise, build the Docker image from this labs directory:
`docker build -t rust-student-core -f lab9-core-dump/docker/Dockerfile .`
Its Docker assets target the student package. Docker and Linux debugger
behavior need a separate rehearsal. The image uses the lockfile in this kit.
