# Reference solution: Production case studies 06–09

Published after the classroom exercises at the trainer's request. This directory contains the worked implementation. Your exercise remains at `production-labs/`.

Run from the participant repository root:

```sh
cargo +nightly-2026-07-22 test --manifest-path solutions/production-labs/Cargo.toml --workspace --offline --locked -j 1
```

The pinned toolchain is required, including for portable SIMD. If dependencies have not been downloaded on this machine, run the same command once without `--offline`. Keep the original acceptance tests intact; passing a reduced or edited test is not equivalent to passing the original suite.

Compare files under `src/` with the exercise and explain each repair. The solutions are runnable independently; no copying over your work is required.
