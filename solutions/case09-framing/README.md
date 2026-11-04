# Lab 09 — worked framing solution (slide 14)

This independent reference repairs only `Decoder::next` in `src/lib.rs`. It preserves header/body bytes and progress across cancellation, validates length before allocation, distinguishes empty frames/clean EOF/truncation, and poisons terminally invalid streams.

`Gateway::apply` intentionally remains the original faulty accounting implementation. Accounting is a later class task. Your original `production-labs/case09-incident` workspace and tests are unchanged.

## Run from the repository root

Baseline infrastructure:

```sh
cargo +nightly-2026-07-22 test --manifest-path solutions/case09-framing/Cargo.toml --test baseline --offline --locked
```

Focused framing regression (exactly one test):

```sh
cargo +nightly-2026-07-22 test --manifest-path solutions/case09-framing/Cargo.toml --test acceptance cancellation_after_every_byte_preserves_frame --offline --locked -- --exact
```

Reproduce the original interrupted-read probe:

```sh
cargo +nightly-2026-07-22 run --manifest-path solutions/case09-framing/Cargo.toml --bin probe --offline --locked -- 1
```

Expect `elapsed=true buffered=6`, then the body `12:100` reconstructed as `[49, 50, 58, 49, 48, 48]`.

## Expected acceptance status

The original `tests/acceptance.rs` is unchanged. Its four framing tests pass:

- `cancellation_after_every_byte_preserves_frame`
- `back_to_back_empty_and_clean_eof`
- `rejects_length_before_body_allocation_and_poisoned_retry`
- `every_truncated_frame_is_distinct_from_clean_eof`

Running the full acceptance suite currently produces **4 passed, 2 failed**: `accounting_is_transactional_on_overflow` and `invalid_frames_do_not_allocate_tenants` remain failing on purpose, pending the accounting lesson. Do not remove assertions or mistake this partial reference for the completed whole lab.

These deterministic duplex-I/O checks are scenario evidence, not a proof for every input, a real-network workload, or a durable retry guarantee. Keep a decoder paired with its original stream.
