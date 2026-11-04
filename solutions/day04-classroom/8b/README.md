# Reference solution: Lab 8B — allocation and retention

Published after the classroom exercises at the trainer's request. This directory contains the worked implementation. Your exercise remains at `day04-classroom/labs/8b/`.

Run from the participant repository root:

```sh
cargo +nightly-2026-07-22 test --manifest-path solutions/day04-classroom/8b/Cargo.toml --workspace --offline --locked -j 1
```

The pinned toolchain is required, including for portable SIMD. If dependencies have not been downloaded on this machine, run the same command once without `--offline`. Keep the original acceptance tests intact; passing a reduced or edited test is not equivalent to passing the original suite.

Compare files under `src/` with the exercise and explain each repair. The solutions are runnable independently; no copying over your work is required.
