# Solution validation — 2026-11-04

Ran on the trainer's macOS ARM64 machine using `nightly-2026-07-22`, checked-in lockfiles, offline dependencies and one Cargo compilation job. Each command used `cargo test --workspace` against the solution manifest.

| Solution suite | Passed test executions, including doc tests | Result |
|---|---:|---|
| `day05-hackathon` | 24 | Passed |
| `day04-classroom/8b` | 10 | Passed |
| `day04-classroom/8c` | 11 | Passed |
| `day04-classroom/final` | 18 | Passed |
| `day04-challenge` | 13 | Passed |
| `production-labs` | 75 | Passed |
| `labs` | 54 | Passed |

The Day 5 hackathon includes **23 acceptance tests and 1 baseline test**. The published hackathon and Lab 8B/8C acceptance-test files match the original student tests byte-for-byte. No weakened tests or single-case replacements were used. Lab 4's compile-fail fixture also passed.

Python runner syntax and argument help were checked. These are native test results, not a fresh Linux/Docker run, Miri run, benchmark, or proof of arbitrary unsafe-code soundness. The earlier real Linux core capture is a separate diagnostic demonstration.
