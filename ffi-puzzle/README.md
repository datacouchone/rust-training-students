# FFI puzzle: borrowed text, invalidation and update boundaries

**Time: 60–75 minutes.** Complete the basic FFI lab first. This standalone
extension provides its completed Rust wrapper, C implementation and Miri mock.
Implement only the two methods marked `PUZZLE` in `src/lib.rs`.

## The foreign contract

Read `csrc/legacy.h` first. `legacy_get` returns database-owned terminated bytes.
They stay valid until replacement of that key or database close. Never free
them. `legacy_put` copies its arguments synchronously. Empty strings are valid
C strings; interior zero bytes are not. A C string need not be valid UTF-8.

## Puzzle A: zero-copy strict lookup

```rust
pub fn get_str<'db>(&'db self, key: &[u8]) -> Result<Option<&'db str>, DbError>
```

- Validate key UTF-8 first, then reject interior NUL. If both are invalid,
  return InvalidUtf8, not InteriorNul.
- Valid missing key → Ok(None). Stored empty value → Ok(Some("")); not absence.
- Return a borrowed view directly into database-owned text, without copying.
- Foreign output must be strict UTF-8. Invalid output → InvalidUtf8, never
  silently replace bytes as the existing owned `get` does.
- The result borrows the database, not the temporary key or local CString.
- Do not call the existing owned get(), allocate a value copy, leak anything,
  cast to 'static, transmute a lifetime, or change the method signature.
- Add a concrete SAFETY comment describing terminator, owner and invalidation.

## Puzzle B: read, transform, then replace

```rust
pub fn update_with<F>(&mut self, key: &[u8], transform: F) -> Result<(), DbError>
where
    F: FnOnce(Option<&str>) -> String,
```

- Build on get_str and the existing safe put API. Add no unsafe code here.
- Invalid keys/output UTF-8 from lookup must be rejected before transform runs.
- For valid lookup, invoke transform exactly once with borrowed old text, or
  None for a missing key. Do not allocate a copy of the old value first.
- The callback returns an owned replacement. End foreign borrows before put
  invalidates the previous storage. Accept callbacks that consume captured data.
- A callback panic must propagate without changing this database's contents.
  Do not catch/swallow the panic and do not write a placeholder before callback.
- Interior NUL in the replacement returns InteriorNul without changing the old
  value or creating a previously missing key. Empty keys/values remain valid.
- Do not change C, the mock, provided methods, signatures or tests. Do not add
  Clone/Copy to the owner or expose a raw handle.

This is not a general transaction: it makes no promise about callback side
effects outside this database, rollback of arbitrary external work, durability,
or thread safety. The callback runs in Rust, not as an extern-C callback.

## Start and validate (participant repository root)

Prerequisites: pinned toolchain, Miri, a working C compiler, and the existing
`cc` crate dependency cache from the main lab. Offline means no network fetch.

First compile everything without executing TODOs:

```sh
cargo +nightly-2026-07-22 test --manifest-path ffi-puzzle/Cargo.toml --no-run --offline
```

After get_str is implemented, save and run these lookup checks:

```sh
cargo +nightly-2026-07-22 test --manifest-path ffi-puzzle/Cargo.toml --test challenge absence_empty_and_unicode_are_distinct --offline
cargo +nightly-2026-07-22 test --manifest-path ffi-puzzle/Cargo.toml --test challenge output_borrows_database_not_temporary_key --offline
```

After both methods are implemented, save and run the complete suite:

The foreign-invalid-UTF-8 unit test checks both lookup and callback suppression,
so it runs at this complete stage, not immediately after lookup alone.

```sh
cargo +nightly-2026-07-22 test --manifest-path ffi-puzzle/Cargo.toml --offline
```

After the normal suite passes, run Miri:

```sh
cargo +nightly-2026-07-22 miri test --manifest-path ffi-puzzle/Cargo.toml --offline
```

Expected completed result: **6 unit tests + 10 integration tests + 4 doctests**.
Three doctests are expected compiler rejections; the fourth must compile and
run. Runtime checks matter: a compile-fail doctest can pass even with TODOs.
Native tests use C; Miri uses the Rust mock, not compiled C. Neither proves all
possible executions sound. Before implementation, TODO failures are expected.

## Reasoning round

Read `TRAPS.md` and predict each result before using the compiler. Explain why
an owned snapshot can survive close but a borrowed view cannot. Identify which
operations allocate and where borrowed output becomes invalid.

## Suggested timing and timer

- 0–10 minutes: read the C contract and predict the trap cases.
- 10–30 minutes: implement strict borrowed lookup and check temporary-key use.
- 30–50 minutes: implement update and investigate NUL/panic behavior.
- 50–60 minutes: full tests, doctests and Miri.
- Optional 15 minutes: explain lifetime constraints and defend each unsafe call.

Open `timer.html` in a browser. Default is 60 minutes; select 75 minutes if you
include the reasoning round. Start/pause/resume/reset/fullscreen work locally.
No server or network is needed for the timer. It restores on refresh when the
browser permits local storage and follows wall time while in a background tab.
