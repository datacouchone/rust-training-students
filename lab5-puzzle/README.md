# Lab 5 challenge: a generic, policy-aware summary derive

**90–120 minutes.** Build WireSummary for named-field structs. Four marked
sections A–D need implementation. Parsing ordinary Rust structure is supplied;
attribute policy, label validation, precise bounds and output generation are yours.

The macro generates an inherent method:

    fn wire_summary(&self) -> String

For a struct named Event with fields id=7 and healthy=true, output exactly:

    Event { id: 7, healthy: true }

Values use Rust Debug formatting in declaration order. Empty output is exactly
"Event {}", with no extra spaces inside the braces. No Clone requirement.

## Field policy

- No helper: show the field's name and Debug value.
- summary(rename="wire_name"): use that label.
- summary(skip): omit the field completely.
- summary(redact): include its label and literal <redacted>; do not evaluate
  its Debug implementation or require its type to implement Debug.
- Rename and redact may combine, including across separate attributes.
- Skip cannot combine with rename or redact.
- Reject duplicate options, unknown options, empty rename strings and wrong
  option syntax. Do not silently ignore malformed policy.
- Final emitted labels must be unique. Skipped fields do not reserve labels.

Helper syntax is an attribute, for example:

    #[derive(lab5_puzzle::WireSummary)]
    struct Login<T> {
        user_id: u32,
        #[summary(rename="credential", redact)]
        secret: T,
    }

Login can summarize a secret that has no Debug implementation at all.
Formatting the secret first and hiding its text later violates the assignment.

## Generic and hygiene requirements

Preserve lifetime, type and const parameters, defaults, and the original where
clause. Add Debug constraints only for visible field types. A skipped or
redacted generic parameter must remain usable without Debug.

Handle empty named structs, raw field identifiers, quotes/braces in renamed
labels, and caller scopes defining their own String/Vec. Use parsed syntax and
quoted tokens, never string-concatenated Rust source. Return source-aware
compile errors; a proc-macro panic is not an acceptable diagnostic.

This is an output-policy teaching model, not a general secret-management system.

## Files

- src/lib.rs: A parse_options, B validate_fields, C bounded_generics,
  D emit_summary. Search the TODO A/B/C/D BEGIN markers.
- tests/baseline.rs: two working setup tests.
- tests/challenge.rs: twelve output/ownership/generic acceptance tests.
- Four unit tests in src/lib.rs: parser, labels and bounds.
- ui/: nine callers that must be rejected, including a visible non-Debug field.
- check_rejections.py: compiler-diagnostic runner.
- HINTS.md, TRAPS.md, timer.html: hints, defense questions and 120-minute timer.

## Commands: run from the participant repository root

If these pinned dependencies are not cached, fetch them while online first:

    cargo +nightly-2026-07-22 fetch --manifest-path lab5-puzzle/Cargo.toml

Then compile and run only the setup tests:

    cargo +nightly-2026-07-22 check --manifest-path lab5-puzzle/Cargo.toml --all-targets --offline
    cargo +nightly-2026-07-22 test --manifest-path lab5-puzzle/Cargo.toml --test baseline --offline

Expect two passed. The starter deliberately emits a compiling method that
panics if called; baseline does not call it. Green compilation is not completion.

### A: parse policy, about 25 minutes

Implement parse_options, including policy spread across multiple attributes.
Save, compile, then run:

    cargo +nightly-2026-07-22 test --manifest-path lab5-puzzle/Cargo.toml --lib parser_ --offline

Expect two selected unit tests passed.

### B: validate final labels, about 15 minutes

Detect collisions after renaming, excluding skipped fields. Save and run:

    cargo +nightly-2026-07-22 test --manifest-path lab5-puzzle/Cargo.toml --lib labels_ --offline

Expect one selected unit test passed.

### C: preserve generics and add precise bounds, about 20 minutes

Add predicates for visible field types without constraining hidden ones. Save:

    cargo +nightly-2026-07-22 test --manifest-path lab5-puzzle/Cargo.toml --lib bounds_ --offline

Expect one selected unit test passed.

### D: generate the method, about 25 minutes

Emit exact formatting, no cloning, no evaluation of redacted values, correct
raw labels and absolute standard-library paths. Save and run:

    cargo +nightly-2026-07-22 test --manifest-path lab5-puzzle/Cargo.toml --test challenge --offline

Expect twelve selected acceptance tests passed.

### Final validation and defense

    cargo +nightly-2026-07-22 test --manifest-path lab5-puzzle/Cargo.toml --offline
    python3 lab5-puzzle/check_rejections.py

Expect four unit + two baseline + twelve acceptance tests, then nine verified
compiler rejections. Zero doctests are normal. This macro puzzle uses host
compiler tests and diagnostic fixtures; its validation commands are native.

Point to one predicate, one emitted formatting operation, and the complete
absence of a redacted field access. Explain why an unsupported input should
produce a helpful compile error, not a proc-macro panic.


## Released advanced reference solution

The [WireSummary solution guide](SOLUTION.md) and [complete solution.rs](solution.rs)
are available separately. The starter remains unfinished. Save your attempt
before applying the reference and running the validation commands.
