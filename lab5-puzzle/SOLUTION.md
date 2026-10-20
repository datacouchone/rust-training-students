# Advanced Lab 5: WireSummary reference solution

[solution.rs](solution.rs) completes all four implementation sections A–D.
This is the advanced policy-aware derive, separate from the main Lab 5
FieldSummary and timed reference in `solutions/lab5-macros/`.
The starter in `src/lib.rs` remains unfinished.

## Run the reference

Save your own attempt somewhere safe before replacing your local implementation.
Run from the participant repository root:

```sh
cp lab5-puzzle/solution.rs lab5-puzzle/src/lib.rs
cargo +nightly-2026-07-22 fetch --manifest-path lab5-puzzle/Cargo.toml
cargo +nightly-2026-07-22 test --manifest-path lab5-puzzle/Cargo.toml --offline
python3 lab5-puzzle/check_rejections.py
```

The copy command replaces your local source. Expect four unit tests, two
baseline tests and twelve acceptance tests: eighteen passing tests total.
The diagnostic runner verifies nine intentionally rejected callers.
This procedural macro is validated through native compiler tests and fixtures.

## Read the implementation

- **A — parse_options:** parse helper attributes, combine rename with redact,
  and reject unknown, duplicate, malformed or conflicting policy.
- **B — validate_fields:** detect final-label collisions after renaming;
  skipped fields do not reserve labels.
- **C — bounded_generics:** preserve the original generics and where clause,
  adding Debug bounds only for fields whose values will be printed.
- **D — emit_summary:** generate the inherent method with quoted tokens,
  exact output formatting and absolute standard-library paths. Redacted
  entries emit a literal marker without accessing or formatting the value.

Inspect the generic and non-Debug redaction tests, then compare each rejected
caller with its compiler diagnostic. Output policy applies only to this method;
it is not a general secret-management guarantee. Tests provide evidence for
these cases, not universal correctness for every possible input.
