# Day 5 worked demos — through slide 14

Completed instructor demonstrations for slides 8–13. The [Lab 09 framing reference](../case09-framing/README.md) completes the framing assignment introduced on slide 14. Earlier slide-10 references remain available separately.

| Slide | Binary | Contract demonstrated |
|---|---|---|
| 8 | `stream_boundaries` | Two complete bodies followed by clean EOF |
| 9 | `cancel_header` | Retain two header bytes through cancellation |
| 10 | `cancel_body` | Retain four header and two body bytes |
| 11 | `truncated_eof` | Distinguish clean EOF, empty frame and truncation |
| 12 | `poisoned_stream` | Reject oversize before allocation; reject poisoned reuse immediately |
| 13 | `every_boundary` | Reconstruct the original body at all eight interior splits |

Each input driver is in `src/bin/<binary>.rs`. Its worked decoder is in `src/scenarios/<binary>.rs`. No participant attempt is overwritten.

From the repository root, select a binary from the table:

```sh
cargo +nightly-2026-07-22 run --manifest-path solutions/day05-through-slide14/Cargo.toml --bin every_boundary --offline --locked
```

These are observation programs, not assertion-based acceptance tests. The Lab 09 reference retains the original acceptance tests. This release does not solve accounting or later exercises.
