# Day 5 worked demos — through slide 10

Reference snapshot of the instructor demonstrations completed on 2026-11-04. This is an independent Cargo workspace; your starter files and Lab 09 attempts stay unchanged.

| Slide | Run target | Input driver | Worked implementation | Expected observation |
|---|---|---|---|---|
| 8 | `stream_boundaries` | `src/bin/stream_boundaries.rs` | `src/scenarios/stream_boundaries.rs` | Two bodies (`1:10`, `2:20`), then clean EOF. |
| 9 | `cancel_header` | `src/bin/cancel_header.rs` | `src/scenarios/cancel_header.rs` | Timeout; progress 2; resumes with `1:100`. |
| 10 | `cancel_body` | `src/bin/cancel_body.rs` | `src/scenarios/cancel_body.rs` | Timeout; retained header + body 6; resumes with `1:100`. |

Each driver has `main()` at line 11. Each scenario has `Decoder::next` at line 31, with its worked body at lines 35–88. The driver supplies fixed input; the scenario implements the reader. Buffers and progress counters belong to the decoder so they survive cancellation of an individual read attempt. Keep each decoder paired with its original stream.

Run from the repository root:

```sh
cargo +nightly-2026-07-22 run --manifest-path solutions/day05-through-slide10/Cargo.toml --bin stream_boundaries --offline --locked
cargo +nightly-2026-07-22 run --manifest-path solutions/day05-through-slide10/Cargo.toml --bin cancel_header --offline --locked
cargo +nightly-2026-07-22 run --manifest-path solutions/day05-through-slide10/Cargo.toml --bin cancel_body --offline --locked
```

These are observation programs. Compare their printed results with the table; a zero process exit alone is not an acceptance assertion. The cancellation probes use Tokio duplex I/O with a paused clock, not a network performance measurement.

Only the three completed demonstrations are included. Slide 7 incident probes remain diagnostic examples in the starter; Lab 09, later demos and the final challenge are not solved by this release.
