# lab1-layout — advanced concept puzzles

Concepts: repr and padding; transparent wrappers; documented vs observed niches.

## Together, then your turn
Measure Header together; students reorder PacketHeader and reason about Message.

Open `starter/src/lib.rs` (lab9 also has `server.rs`, `state.rs`, and `frame.rs`).
Read contracts and tests, then work on one TODO at a time. Existing scaffolding
is deliberate: only the concept under discussion should need completion.
For lab9, the puzzle is investigation and repair of faults rather than filling
every working server function. Record reproduction, evidence and root cause.

Run from `students/labs/`:

```sh
cargo test -p lab1-layout-starter -- --ignored
```

Done: test_optimized_header_size, test_repr_c_layout, test_message_size_after_boxing; include ignored tests. Use the pinned nightly. Unfinished TODOs can cause test
panics; they are expected. This card is a puzzle sequence, not permission to
skip the safety reasoning or operational prerequisites.

For lab9's core exercise, build the Docker image from this labs directory:
`docker build -t rust-student-core -f lab9-core-dump/docker/Dockerfile .`
Its Docker assets target the student package. Docker and Linux debugger
behavior need a separate rehearsal. The image uses the lockfile in this kit.
