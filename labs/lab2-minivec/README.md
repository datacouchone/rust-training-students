# Lab 2 — MiniVec: a safe abstraction over raw allocation

## Goal

Implement `MiniVec<T>`, a stripped-down `Vec<T>`, by hand-managing a raw
heap allocation. By the end you should be able to:

- allocate, grow, and free memory directly with `std::alloc::{alloc,
  realloc, dealloc}` and `Layout`, instead of relying on a collection type
  to do it for you;
- explain why `MiniVec` stores `ptr: NonNull<T>` + `_marker: PhantomData<T>`
  instead of just a raw `*mut T`, in terms of variance and drop-check;
- write `unsafe` blocks that each carry a real, specific safety argument
  (not "trust me") tied to the invariants your own code establishes; and
- check tested executions with Miri and justify the safety invariants;
  a clean run does not prove soundness for all inputs or callers.
  See [Miri limitations](https://github.com/rust-lang/miri#readme).

This is the same problem worked through in the Rustonomicon's
"Implementing Vec" chapter — this lab is that exercise, scoped down and
turned into a from-scratch fill-in-the-blanks with a test suite that
pins down the tricky parts (growth, drop, ZSTs).

## Time box

**60–75 minutes.** Roughly: 10 min reading the starter + this README,
35–45 min implementing and getting `cargo test` green, 10–15 min running
Miri and fixing anything it flags, remaining time for stretch goals.

## Starter entry point

`starter/src/lib.rs`. The struct and every method signature are already
there, matching this contract:

```rust
pub struct MiniVec<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
    _marker: PhantomData<T>,
}
```

with `new, with_capacity, push, pop, get, len, capacity`, plus
`Deref<Target = [T]>` and `Drop`.

`new()` is fully implemented for you (it's the one method that needs no
`unsafe`, since an empty `MiniVec` never allocates — `ptr` is set to
`NonNull::dangling()`). Every other method body is `todo!()`. Wherever
you're about to write an `unsafe` block, you'll find a
`// SAFETY: <fill in>` comment scaffold above the relevant TODO — replace
`<fill in>` with the actual argument for why that operation is sound,
before you write the block. The starter's own test module (bottom of the
file) is the exact suite your implementation needs to pass; run
`cargo test` as you go and let the panics guide you.

## Steps

1. **`with_capacity(capacity)`** — build a `Layout::array::<T>(capacity)`,
   call `alloc::alloc`, and check the returned pointer isn't null before
   wrapping it in `NonNull` (use `alloc::handle_alloc_error` if it is).
   Don't forget the ZST guard (see `new`) and the `capacity == 0` case.

2. **`grow`** (private helper) — this is where the "doubling" strategy
   lives: go from `cap == 0` to `cap == 1` with `alloc::alloc`, and from
   `cap == n` to `cap == 2n` with `alloc::realloc`, passing the *old*
   `Layout` `realloc` needs to know what it's resizing from.

3. **`push`** — call `grow()` when `len == cap`, then `ptr::write` the new
   value into the slot at offset `len` (not a plain store — the slot is
   uninitialized memory, and a plain assignment would try to drop
   whatever garbage was already "in" it) and bump `len`.

4. **`pop`** — the mirror image: decrement `len` *first*, then
   `ptr::read` the value out of the slot that index now names. Getting
   the order right (decrement, then read) is what prevents a double-drop
   if the caller drops the returned value and then something else reads
   the same slot.

5. **`get`** — bounds-check against `len`, then dereference.

6. **`Deref`** — build a `&[T]` over `[ptr, ptr + len)` with
   `slice::from_raw_parts`. This is what makes `.iter()`, indexing,
   `assert_eq!(&v[..], &[...])`, etc. all work on `MiniVec` for free.

7. **`Drop`** — the easiest one to get subtly wrong. Two jobs, in order:
   drop every *live* element (`[0, len)`) exactly once, then (only if
   `cap != 0` — a never-allocated `MiniVec` must not call `dealloc` on a
   dangling pointer) free the backing buffer using the *same* `Layout` it
   was allocated with.

8. Run `cargo test -p lab2-minivec-starter` until all five tests are green.

9. Run Miri — this is not optional, it's the actual point of the lab:

   ```bash
   cargo +nightly-2026-07-22 miri test -p lab2-minivec-starter
   ```

   A green `cargo test` with a red Miri run means you have a bug (stacked
   borrows violation, use-after-free, wrong `Layout` passed to
   `dealloc`/`realloc`, uninitialized-memory read, etc.) that ordinary
   testing didn't catch. Read the Miri diagnostic — it names the specific
   allocation and access that's the problem.

## Done-criteria

- [ ] `cargo test -p lab2-minivec-starter` — all 5 tests pass:
      `test_push_pop`, `test_growth_amortization`,
      `test_drop_called_exactly_once`, `test_deref_slice`,
      `test_zst_rejected`.
- [ ] `cargo +nightly-2026-07-22 miri test -p lab2-minivec-starter` —
      passes with no UB reported.
- [ ] Every `unsafe` block has a `// SAFETY:` comment that states the
      actual invariant making it sound (references `len`/`cap`/`ptr`
      specifically — not a restatement of what the line of code does).
- [ ] `cargo build -p lab2-minivec-starter` compiles (bodies may still be
      `todo!()`).

## Stretch goals

- Implement `IntoIterator` for `MiniVec<T>` (by value) — this needs its
  own small iterator struct that owns the allocation and `ptr::read`s
  elements out one at a time, then frees the buffer without dropping
  elements again in *its* `Drop` impl. Compare it to how `std::vec::IntoIter`
  does the same thing.
- Add `insert(index, value)` and `remove(index)`, which need
  `ptr::copy`/`ptr::copy_nonoverlapping` to shift the tail of the buffer.
- Make `MiniVec` support ZSTs correctly instead of rejecting them: for a
  ZST, `cap` should behave as if it's always `usize::MAX` and you should
  never call the allocator at all (pointer arithmetic on a ZST pointer is
  a no-op). This is the part of `std::vec::Vec`'s real implementation
  that this lab intentionally cuts.
- Shrink the allocation in `pop` once `len` drops far below `cap` (e.g.
  below `cap / 4`), and write a test asserting `capacity()` actually goes
  down — think about whether this helps or hurts the amortized-growth
  argument from `test_growth_amortization`.
- Try deliberately introducing a bug (e.g. swap the order of the
  decrement and the read in `pop`, or forget the `cap != 0` guard in
  `Drop`) and confirm Miri actually catches it. This is worth doing once:
  it's the difference between "Miri is a formality" and "Miri is why I'd
  trust this code."
- Handle panicking element `Drop` without leaking the buffer: right now
  if an element's destructor panics inside `MiniVec::drop`, the remaining
  elements are still dropped correctly (no double-drop), but unwinding
  skips the `dealloc` call, so the backing allocation leaks. Fix it the
  way `std::vec::Vec` does — split the allocation out into its own
  `RawVec`-style type with its own `Drop` impl, so that type's destructor
  still runs (and frees the buffer) while unwinding past a panicking
  element drop.
