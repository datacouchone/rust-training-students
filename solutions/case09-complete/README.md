# Lab 09 — complete worked solution (through slide 18)

This independent workspace combines the completed framing decoder with transactional tenant accounting. This is the instructor-led reference for slide 18. Original participant attempts and the earlier framing-only reference remain unchanged.

## Run from your cloned repository root

```sh
git pull --ff-only origin main
cargo +nightly-2026-07-22 test --manifest-path solutions/case09-complete/Cargo.toml --offline --locked
```

All six original acceptance tests and the baseline must pass. No test assertions were removed or changed.

## Before and after walkthrough

- Before: [framing-only source](../case09-framing/src/lib.rs#L118). Decoder repaired; accounting still has two expected acceptance failures.
- After: [complete source](src/lib.rs#L118). Same decoder and tests; Gateway::apply now validates before committing.
- Checks: [original acceptance tests](tests/acceptance.rs#L66).

| Lines in complete src/lib.rs | Meaning |
|---|---|
| 119–122 | Validate UTF-8, split tenant:bytes and parse both numbers before changing state. |
| 123–125 | Reject a new tenant if the map is full; existing tenants may still update. |
| 126 | Clone the old account, or create a local empty candidate without inserting it. |
| 127–130 | Compute both new fields with checked arithmetic. An error leaves the map unchanged. |
| 131 | Commit the complete account only after all rejection checks pass. |
| 132 | Return the accepted result. |

## Replay the incident probes

```sh
cargo +nightly-2026-07-22 run --manifest-path solutions/case09-complete/Cargo.toml --bin probe --offline --locked -- 1
cargo +nightly-2026-07-22 run --manifest-path solutions/case09-complete/Cargo.toml --bin probe --offline --locked -- 2
cargo +nightly-2026-07-22 run --manifest-path solutions/case09-complete/Cargo.toml --bin probe --offline --locked -- 3
```

1. A timeout preserves six consumed bytes; retry reconstructs 12:100.
2. Overflow returns an error and leaves accepted=1 and bytes=u64::MAX unchanged.
3. Capacity two admits tenants 0 and 1, then rejects tenants 2, 3 and 4 with Cardinality.

The tests cover their specified scenarios. This in-memory commit pattern does not provide crash durability or distributed transactions.
