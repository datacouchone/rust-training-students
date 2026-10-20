# Lab 5 reference: derive and timing macros

This independent crate contains the complete main Lab 5 implementation:

- FieldSummary generates field names and written types, honoring summary(skip).
- timed preserves the demonstrated synchronous return values and early returns.
- Its async branch preserves await and measures elapsed time once execution
  reaches the wrapper during polling.

The unfinished assignment remains in labs/lab5-derive-macro/starter. This
reference does not replace it or include the separate WireSummary challenge.

## Run from the participant repository root

If dependencies are not cached yet, fetch them while online:

```sh
cargo +nightly-2026-07-22 fetch --manifest-path solutions/lab5-macros/Cargo.toml
```

Run all reference tests:

```sh
cargo +nightly-2026-07-22 test --manifest-path solutions/lab5-macros/Cargo.toml --offline
```

Expect three parser unit tests and five integration tests to pass. None are
ignored. Zero doctests is normal.

Run the small consumer containing both macros:

```sh
cargo +nightly-2026-07-22 run --manifest-path solutions/lab5-macros/Cargo.toml --example timed_demo --offline
```

The summary is `host: String, port: i32`. The three function results are 43,
negative, and 42. Timing messages go to stderr and their durations vary.

## Generate the expanded Rust

This first command creates the output directory even on a fresh clone. The
shell opens a redirected output file before Cargo starts, so keep this step.

```sh
mkdir -p solutions/lab5-macros/target
```

```sh
cargo +nightly-2026-07-22 rustc --manifest-path solutions/lab5-macros/Cargo.toml --example timed_demo --offline -- -Zunpretty=expanded > solutions/lab5-macros/target/expanded-timed.rs
```

Open solutions/lab5-macros/target/expanded-timed.rs in VS Code. Search for:

| Search | Inspect |
|---|---|
| impl FieldSummary for Config | Generated derive implementation and skipped api_key |
| fn compute( | Start instant, original body, elapsed measurement, print, return |
| fn classify( | Early returns inside the generated inner closure |
| async fn fetch( | Async block and await inside the generated async function |

For compute, read the generated statements in this order: start the clock,
execute the closure, store its answer, calculate elapsed time, print, return.
The final pair of parentheses after the closure invokes it immediately.

Expansion writes source for inspection; it does not run the example or measure
anything. The run command executes the functions. Edit the original source,
then regenerate rather than editing the expanded file.

## Generate the same view from an unchanged starter

The starter already supplies timed, so students can inspect it before solving
the derive exercise:

```sh
mkdir -p labs/target
```

```sh
cargo +nightly-2026-07-22 rustc --manifest-path labs/Cargo.toml -p lab5-derive-macro-starter --test expand --offline -- -Zunpretty=expanded > labs/target/lab5-expanded.rs
```

Open labs/target/lab5-expanded.rs and search for fn compute(, fn classify( or
async fn fetch(. The unfinished summary function may still contain a panic;
expanding the code does not call that function.

## Scope

The main derive supports named, non-generic structs and describes declarations,
not stored field values. Emitted standard-library paths are absolute. The timing
wrapper logs normal completion; panic or async cancellation can skip its trailing
log. Elapsed wall time can include waiting and is not CPU-time measurement.
Use the advanced WireSummary puzzle for generic parameters and richer policy.

Generated target files are build artifacts and do not need committing.
