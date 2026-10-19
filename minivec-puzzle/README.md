# MiniVec puzzle: insertion, removal and ownership

**Time: 60 minutes.** Build on the completed MiniVec without changing its existing methods.

Open `src/lib.rs`. Implement only `insert` and `remove`, each marked `PUZZLE`.
All other MiniVec methods and the five original unit tests are provided.

## Requirements

### Insert

`insert(index, value)` inserts before index, preserving element order.

- Accept `0..=len`; inserting at len appends. Empty insertion at zero is valid.
- Panic for index > len **before growing or changing collection state**.
- Grow only if full, using the provided grow method.
- Shift the suffix right within the allocation, then initialize the new slot.
- Increase length exactly once. Support non-Copy, non-Clone values.

Example: `[10, 30, 40]` → `insert(1, 20)` → `[10, 20, 30, 40]`.

### Remove

`remove(index) -> T` transfers the selected element to the caller.

- Accept only index < len; panic otherwise, without changing collection state.
- Preserve the order of the remaining elements by shifting the suffix left.
- Decrease length exactly once. Do not shrink or allocate storage.
- Do not drop the returned element. Exclude its former ownership and stale
  trailing bits from the collection's final live range.

Example: `[10, 20, 30, 40]` → `remove(1)` returns `20`, leaving `[10, 30, 40]`.

## Constraints and safety argument

Keep the existing public signatures. Do not use a temporary Vec, require Clone
or Copy, change tests, or implement operations by repeatedly popping and pushing.
Use raw-pointer operations and explain each unsafe block in a SAFETY comment.

Explain overlap, initialization, allocation bounds and ownership. Pointer
addresses obtained before growth must not be reused after reallocation.
At public method boundaries, len <= cap and exactly the first len slots own
initialized values. Spare slots must never be read or dropped as live values.

Zero-sized types remain unsupported. The provided normal cleanup can leak the
backing allocation if an element destructor panics; fixing that is outside this
challenge. No element destructors should run merely because a value is shifted.

## Run from the participant repository root

First compile the starter and tests; these commands do not execute TODOs:

```sh
cargo +nightly-2026-07-22 check --manifest-path minivec-puzzle/Cargo.toml --offline
cargo +nightly-2026-07-22 test --manifest-path minivec-puzzle/Cargo.toml --no-run --offline
```

After implementing insert, save and run its first focused check:

```sh
cargo +nightly-2026-07-22 test --manifest-path minivec-puzzle/Cargo.toml --test challenge insert_positions_and_growth --offline
```

After implementing both methods, save and run all eight challenge tests:

```sh
cargo +nightly-2026-07-22 test --manifest-path minivec-puzzle/Cargo.toml --test challenge --offline
```

After those pass, run the full suite and then Miri:

```sh
cargo +nightly-2026-07-22 test --manifest-path minivec-puzzle/Cargo.toml --offline
cargo +nightly-2026-07-22 miri test --manifest-path minivec-puzzle/Cargo.toml --offline
```

Expected completed result: **5 original unit tests + 8 challenge tests** pass.
Before implementation, runtime challenge failures at TODOs are expected. Passing
the invalid-index panic tests alone does not validate either implementation.
Miri checks exercised Rust executions; it is not proof for every possible input.

## Suggested timing

- 0–10 minutes: predict layouts and draw both overlapping shifts.
- 10–30 minutes: implement insert; test beginning, middle, end and growth.
- 30–45 minutes: implement remove and ownership transfer.
- 45–60 minutes: run the full suite and Miri; explain safety arguments.

## Countdown timer

Open `timer.html` in a browser. It runs locally without a server or network.
Choose 45, 60 or 75 minutes, or enter a custom duration. Start, pause/resume,
reset and fullscreen controls are provided. The timer follows elapsed wall
time when the tab is in the background and restores after a refresh in the
same browser when local storage is available. It does not play sound.

## Optional hint

Compare `std::ptr::copy` with `copy_nonoverlapping`: which permits overlapping
source and destination ranges? Moving bytes alone does not establish ownership;
the initialized range and length updates complete that argument.
