# lab9-core-dump: crash/debug environment.
#
# Builds the student binary (same faulty code as starter, see
# ../README.md) plus the tools needed to reproduce and post-mortem the
# `deadlock` fault with a real core dump: gdb, the pinned nightly
# toolchain, and GDB for a live core snapshot.
#
# Build (trainer machine, requires Docker), from the `labs/` workspace
# root so the build context can see the workspace manifest/lockfile:
#   cd labs
#   docker build -t lab9-core-dump -f lab9-core-dump/docker/Dockerfile .
#
# Only the files this image actually needs are COPYed in below (root
# manifests + `lab9-core-dump/` + `prereq-check/`, which is a *non-glob*
# workspace member and so must exist for the workspace to resolve at
# all) -- deliberately not `COPY . .`, both to avoid dragging in the
# (potentially huge, gitignored) `target/` directory and to keep this
# lab's Docker assets self-contained without needing a workspace-root
# `.dockerignore`.
#
# Run from the Mac after Docker Desktop is running:
#   docker run -it --cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
#     --name lab9-crash lab9-core-dump
# No --privileged or kernel-global core_pattern change is needed.
# Keep the named container until its core, matching binary and logs are copied.
# End-to-end capture must be rehearsed on the delivery machine.

FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    CARGO_TARGET_DIR=/workspace/target

# Base tools + GDB for capture/analysis + Python for the load client.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        build-essential \
        gdb \
        procps \
        binutils \
        python3 \
        netcat-traditional \
    && rm -rf /var/lib/apt/lists/*

# Pinned nightly toolchain -- must match ../../rust-toolchain.toml exactly
# so the binary built here behaves identically to the one attendees run
# locally.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
        sh -s -- -y --default-toolchain nightly-2026-07-22 --profile minimal \
        --component rust-src

# The core snapshot uses GDB; tokio-console is a separate live-observation
# workflow and is not needed in this image.
WORKDIR /workspace
# Narrow COPY -- see the build-context note above for why this isn't
# `COPY . .`. `prereq-check` is required even though this lab doesn't use
# it: it's a literal (non-glob) entry in `[workspace] members`, so Cargo
# needs its manifest present to resolve the workspace at all.
COPY Cargo.toml Cargo.lock rust-toolchain.toml /workspace/
COPY prereq-check /workspace/prereq-check
COPY lab9-core-dump/starter /workspace/lab9-core-dump/starter
# Preserve every workspace member so --locked retains the checked-in resolution.
COPY lab1-layout/starter /workspace/lab1-layout/starter
COPY lab2-minivec/starter /workspace/lab2-minivec/starter
COPY lab3-ffi/starter /workspace/lab3-ffi/starter
COPY lab4-typestate/starter /workspace/lab4-typestate/starter
COPY lab5-derive-macro/starter /workspace/lab5-derive-macro/starter
COPY lab6-atomics/starter /workspace/lab6-atomics/starter
COPY lab7-executor/starter /workspace/lab7-executor/starter
COPY lab8-simd-perf/starter /workspace/lab8-simd-perf/starter

# Debug info in release builds is already enabled at the workspace level
# ([profile.release] debug = true in labs/Cargo.toml) -- that's what
# makes the gdb walkthrough in analyze.md legible (symbol names, not just
# addresses).
RUN cargo build -j 1 --locked --release -p lab9-core-dump-starter --bin lab9-core-dump-starter

COPY lab9-core-dump/docker /workspace/lab9-core-dump/docker
WORKDIR /workspace/lab9-core-dump/docker
RUN chmod +x run-crash.sh

CMD ["./run-crash.sh"]
