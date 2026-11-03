# Lab 8A — released routing index solution

Released by the instructor after Lab 8A, through Day 4 slide 16 (2026-11-03).
This independent crate contains the worked `Index::compile` and `Index::route`
implementation and the unchanged public tests. Your original attempt stays in
`production-labs/case08a-profile`.

From the repository root:

```sh
cargo +nightly-2026-07-22 test --manifest-path solutions/case08a-profile/Cargo.toml --offline --locked
cargo +nightly-2026-07-22 run --release --manifest-path solutions/case08a-profile/Cargo.toml --bin profile-route --offline --locked -- compare 10000 256
```

`compile` validates and owns the rules. `route` searches from the full path toward
root at slash boundaries. At each prefix, the tenant-specific destination wins
over the shared destination; a longer applicable prefix wins over a shorter one.
No matching destination is `None`; pool zero is a valid destination.

See `tests/acceptance.rs` for the actual rules, request paths and assertions.
The slide 10 test checks segment boundaries and longest-prefix selection. The
slide 11 test checks tenant ties and absence. Tests check behavior; the comparison
runner reports costs separately, including index construction. Keep the raw
measurements and avoid treating one workload as a universal performance claim.

See [the original lab contract](../../production-labs/case08a-profile/README.md)
and [the slide 16 release guide](../../DAY-04-THROUGH-SLIDE-16.md).
