# Day 4 — worked code through slide 16

Released 2026-11-03. Lab 8A is complete; later exercises remain separate.

| Slide | Worked code | What to observe |
| --- | --- | --- |
| 8 | `day04-production/src/bin/routing_owned.rs` | Owned snapshot acquired with ArcSwap `load_full`. |
| 10–11, 14 | `solutions/case08a-profile/src/lib.rs` and `tests/acceptance.rs` | Segment matching, longest applicable prefix, tenant ties, validation and owned configuration. |
| 9, 15 | `solutions/case08a-profile/src/bin/profile-route.rs` | Stage timing and separate index construction cost. |
| 12 | `day04-production/src/bin/routing_reload.rs` | Existing request retains generation 1 after generation 2 is published; invalid update is rejected. |
| 13 | `day04-production/src/bin/routing_guard.rs` | Short synchronous lookup using an ArcSwap guard. |
| 16 | `day04-production/src/bin/encode_reuse.rs` | Reuse retains output capacity, including after a burst. |

## Run the demonstrations

From the repository root, substitute the binary name from the table:

```sh
cargo +stable run --release --manifest-path day04-production/Cargo.toml --bin encode_reuse --offline --locked
```

For Lab 8A:

```sh
cargo +nightly-2026-07-22 test --manifest-path solutions/case08a-profile/Cargo.toml --offline --locked
```

The original `production-labs/case08a-profile` attempt is not overwritten.
The four small demo binaries above now contain the worked variants. Their
benchmarks run all implementations independently of the saved demo variant.

## Why the encoder immediately calls a closure

Read the comments above `Encoder::encode` in `day04-production/src/encoding.rs`.
`(|| { ... })()` runs a no-argument closure immediately on the same thread.
The `?` operators return an error from the closure into `result`. The outer
function can then reset partial output before returning the error. Without this
boundary, those `?` operators would return directly from `encode`, skipping that
cleanup. Only a successful complete batch is returned as a borrowed byte slice.

The input comes from `encoding::fixture`, not a file. Encoding writes NDJSON
into memory; these examples do not send it over a network. A configured timeout
is not measured latency, retained capacity is not process RSS, and an equal byte
count alone does not establish byte-for-byte output equality.
