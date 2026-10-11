# lab9-core-dump — advanced concept puzzles

Concepts: Result vs panic; task lifecycle; lock ordering; logs; core evidence.

## Together, then your turn
Investigate one injected fault together; students diagnose the remaining faults and propose fixes.

Open `starter/src/lib.rs` (lab9 also has `server.rs`, `state.rs`, and `frame.rs`).
Read contracts and tests, then work on one TODO at a time. Existing scaffolding
is deliberate: only the concept under discussion should need completion.
For lab9, the puzzle is investigation and repair of faults rather than filling
every working server function. Record reproduction, evidence and root cause.

Run from `students/labs/`:

```sh
cargo test -p lab9-core-dump-starter
```

Done: reproduction plus evidence and regression checks; Docker walkthrough is a separate check. Use the pinned nightly. Unfinished TODOs can cause test
panics; they are expected. This card is a puzzle sequence, not permission to
skip the safety reasoning or operational prerequisites.

For lab9's core exercise, build the Docker image from this labs directory:
`docker build -t rust-student-core -f lab9-core-dump/docker/Dockerfile .`
Its Docker assets target the student package. Docker and Linux debugger
behavior need a separate rehearsal. The image uses the lockfile in this kit.

### Linux core snapshot

Docker Desktop runs a Linux VM on your Mac. The lab container builds a
native Linux binary for that VM (Apple Silicon uses ARM64 by default).
Run these commands from the repository root, after starting Docker Desktop:

```bash
docker info
cd students/labs
docker build -t lab9-core-dump -f lab9-core-dump/docker/Dockerfile .
docker run -it --cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
  --name lab9-crash lab9-core-dump
```

The entrypoint runs the intentionally hung server, requires a request timeout,
then uses GDB to capture a **live core snapshot**. It stops only that lab server
and leaves you in the container's Linux shell. It does not change the Linux
VM's global `core_pattern` and does not require `--privileged`.

Follow [the GDB walkthrough](docker/analyze.md). A timeout and parked workers
are evidence to correlate with logs, not sufficient proof of deadlock alone.
From a second Mac terminal, preserve the exact binary, snapshot and logs:

```bash
mkdir -p lab9-artifacts
docker cp lab9-crash:/tmp/lab9-crash lab9-artifacts/
docker cp lab9-crash:/workspace/target/release/lab9-core-dump-starter lab9-artifacts/
```

Exit the Linux shell when finished, then run `docker rm lab9-crash` from the
Mac. No ports need publishing: the load client runs inside the container.
Keep GDB analysis inside Linux; the binary and core are Linux artifacts.

If the daemon is unavailable, start Docker Desktop and retry `docker info`.
If attach is denied, check the capability/security options above and Docker
Desktop's isolation policy; use a dedicated Linux VM if that policy forbids
ptrace. The script's `capture.log` records the actual GDB error. Read the
walkthrough alongside live logs if Docker is temporarily unavailable, but
record that as a fallback rather than a successful core-capture rehearsal.


