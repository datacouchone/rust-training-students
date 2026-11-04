//! Lab 8 — SIMD + arena allocation + profiling.
//!
//! Scenario: a CPU-bound "log-metrics scorer". Each log record contributes
//! four `f32` metrics — `[latency, cpu, mem, err_rate]` — packed back to
//! back into one flat `&[f32]` (so a batch of `n` records is `4 * n` floats
//! long). The scorer normalizes each metric into roughly `[0, 1]`, clamps
//! it (bad/garbage readings can be negative or far above the expected
//! range), and folds it into a weighted sum where the weight cycles through
//! the four metric columns.
//!
//! [`score_scalar`] is the naive, branchy, per-element baseline you'd write
//! without thinking about vectorization. [`score_simd`] is the same
//! computation expressed with `std::simd::f32x8`, processing 8 floats per
//! iteration with a scalar tail for the remainder. [`Arena`] is a small bump
//! allocator used by [`parse_batch`] to avoid heap-allocating each parsed
//! [`LogRecord`] individually.
#![feature(portable_simd)]

use std::alloc::{alloc, dealloc, Layout};
use std::cell::Cell;
use std::ptr::NonNull;
use std::simd::prelude::*;

/// Metrics are expected in roughly the `0..100` range; this scales them
/// down to roughly `0..1` before clamping.
const NORM: f32 = 1.0 / 100.0;

/// Per-column importance weights, cycled `latency, cpu, mem, err_rate,
/// latency, cpu, mem, err_rate, ...` across the flattened input.
const WEIGHTS: [f32; 4] = [0.4, 0.3, 0.2, 0.1];

/// Naive scalar scorer: normalize → clamp → weighted-sum, one element at a
/// time. Deliberately written the way most people would write it first —
/// a per-element modulo/index into `WEIGHTS`, `max`/`min` clamping, and no
/// vectorization — so it makes a good baseline to profile and then beat
/// with [`score_simd`].
pub fn score_scalar(values: &[f32]) -> f32 {
    let mut total = 0.0f32;
    for (i, &v) in values.iter().enumerate() {
        let normalized = v * NORM;
        let clamped = normalized.clamp(0.0, 1.0);
        total += clamped * WEIGHTS[i % WEIGHTS.len()];
    }
    total
}

/// SIMD scorer: identical math to [`score_scalar`], but processes 8 `f32`s
/// per loop iteration with `std::simd::f32x8`, then finishes any leftover
/// elements (0..7 of them) with the scalar path.
///
/// The weight pattern has period 4 and every SIMD chunk is 8 lanes wide, so
/// each full chunk always starts on a multiple of 4 in the weight cycle —
/// meaning a single constant lane-weight vector
/// `[w0, w1, w2, w3, w0, w1, w2, w3]` is correct for *every* chunk, with no
/// gather/index-per-lane needed.
pub fn score_simd(values: &[f32]) -> f32 {
    const LANES: usize = 8;

    let norm = f32x8::splat(NORM);
    let zero = f32x8::splat(0.0);
    let one = f32x8::splat(1.0);
    // WEIGHTS repeats twice per 8-lane chunk: [w0,w1,w2,w3,w0,w1,w2,w3].
    let weights = f32x8::from_array([
        WEIGHTS[0], WEIGHTS[1], WEIGHTS[2], WEIGHTS[3], WEIGHTS[0], WEIGHTS[1], WEIGHTS[2],
        WEIGHTS[3],
    ]);

    let mut acc = f32x8::splat(0.0);
    let (chunks, remainder) = values.as_chunks::<LANES>();

    for &chunk in chunks {
        let v = f32x8::from_array(chunk);
        let normalized = v * norm;
        let clamped = normalized.simd_clamp(zero, one);
        acc += clamped * weights;
    }

    let mut total = acc.reduce_sum();

    // Scalar tail. `tail_start` is `values.len()` rounded down to a
    // multiple of `LANES` (== 8), which is also a multiple of
    // `WEIGHTS.len()` (== 4), so `WEIGHTS[(tail_start + offset) % 4]` lines
    // up exactly with what `score_scalar` would compute for these same
    // global indices.
    let tail_start = values.len() - remainder.len();
    for (offset, &v) in remainder.iter().enumerate() {
        let i = tail_start + offset;
        let normalized = v * NORM;
        let clamped = normalized.clamp(0.0, 1.0);
        total += clamped * WEIGHTS[i % WEIGHTS.len()];
    }

    total
}

/// Base alignment for the arena's backing allocation. 64 bytes covers every
/// common scalar/SIMD type (up to a cache line) with zero padding for the
/// very first allocation; anything with a larger alignment requirement is
/// still handled correctly (just possibly with some padding), since
/// [`Arena::alloc`] computes padding from the *actual* runtime address on
/// every call rather than assuming the base alignment is sufficient.
const ARENA_ALIGN: usize = 64;

/// A single-threaded bump ("arena") allocator.
///
/// # Design: why `alloc(&self, ..) -> &T` instead of `&mut self`
///
/// A `Vec`-backed allocator would need `&mut self` to hand out a new
/// element, because growing/pushing can move existing data and because two
/// live `&mut` `Vec` accesses can't safely overlap. An arena never moves or
/// reuses bytes it has already handed out (until [`Arena::reset`]), so two
/// *different* allocations never alias each other — only the *bump cursor*
/// is shared, mutable state. That's exactly what [`Cell`] is for: `alloc`
/// takes `&self` and mutates only the `Cell<usize>` cursor through interior
/// mutability, while the freshly written `T` gets its own, non-overlapping
/// slice of the backing buffer.
///
/// Taking `&self` (rather than `&mut self`) is also what makes the arena
/// usable the way real arenas are used: allocate many objects that all
/// borrow from the same arena at the same time (e.g. build up a graph of
/// cross-referencing nodes), which `&mut self` per-allocation would forbid
/// (you can't have two live `&mut self` borrows).
///
/// The returned `&T` borrows `self` (`fn alloc<T>(&self, val: T) -> &T` is
/// sugar for `fn alloc<'a, T>(&'a self, val: T) -> &'a T`), which ties the
/// reference's lifetime to the arena. That's what makes [`Arena::reset`]
/// safe to expose as an ordinary safe `&mut self` method: the borrow
/// checker will not let you call `reset` (or drop the arena) while any
/// `&T` obtained from `alloc` is still alive, because `reset` needs
/// exclusive access and an outstanding `&T` is a live shared borrow of
/// `self`. No manual bookkeeping or `unsafe` is needed at the call site to
/// get that guarantee — it's the ordinary aliasing rules, just applied to
/// a hand-rolled allocator instead of a `Vec`.
///
/// # Design: `reset` does not run `Drop`
///
/// The arena is untyped after `alloc` returns (bytes only) — it has no
/// record of what type was written at a given offset, so `reset` cannot
/// call `T::drop`. It simply rewinds the bump cursor to 0, so a subsequent
/// `alloc` will overwrite those bytes with a new value. For `Copy`/POD data
/// (the intended use case — see [`parse_batch`]/[`LogRecord`]) this is
/// invisible; for a `T` with a non-trivial `Drop` impl, resetting (or
/// dropping the arena) will *leak* whatever that destructor would have
/// freed, rather than causing any memory unsafety. This is the same
/// trade-off made by general-purpose bump allocators like `bumpalo`.
pub struct Arena {
    /// Base pointer of the single backing allocation.
    buf: NonNull<u8>,
    /// Usable size of the backing allocation, in bytes.
    capacity: usize,
    /// Layout the backing allocation was made with (needed to `dealloc` it
    /// correctly in `Drop`).
    layout: Layout,
    /// Bump cursor: number of bytes from `buf` already handed out
    /// (including any alignment padding). Behind a `Cell` so `alloc` can
    /// mutate it while only borrowing `&self` — see the struct docs.
    offset: Cell<usize>,
}

impl Arena {
    /// Creates a new arena with at least `capacity` bytes of usable space.
    pub fn new(capacity: usize) -> Self {
        // `Layout` requires a nonzero size to be meaningful for `alloc`; a
        // zero-capacity arena is still valid to construct (it will simply
        // fail the very first `alloc` call), so round up to 1 byte.
        let size = capacity.max(1);
        let layout = Layout::from_size_align(size, ARENA_ALIGN)
            .expect("Arena capacity overflows an allocation Layout");

        // SAFETY: `layout` has a nonzero size and a valid (nonzero,
        // power-of-two) alignment, which is exactly what `alloc`'s contract
        // requires. We check the returned pointer for null below instead of
        // assuming success.
        let ptr = unsafe { alloc(layout) };
        let buf = match NonNull::new(ptr) {
            Some(p) => p,
            None => std::alloc::handle_alloc_error(layout),
        };

        Arena {
            buf,
            capacity: size,
            layout,
            offset: Cell::new(0),
        }
    }

    /// Bump-allocates space for `val`, writes it in, and returns a
    /// reference to it that lives as long as the arena is borrowed.
    ///
    /// Panics if the arena does not have enough remaining space (including
    /// any padding needed to align `T`).
    pub fn alloc<T>(&self, val: T) -> &T {
        let layout = Layout::new::<T>();
        let cur = self.offset.get();

        // SAFETY: `cur` is maintained as an invariant to always be
        // `<= self.capacity` (checked before every `offset.set` below, and
        // reset to 0 by `reset`), so `buf + cur` is within one-past-the-end
        // of the single allocation from `Arena::new`, which is exactly what
        // `add` requires.
        let current: *mut u8 = unsafe { self.buf.as_ptr().add(cur) };

        // Bytes of padding needed so `current + pad` is aligned for `T`.
        // `align_offset` on a byte pointer computed from a real runtime
        // address always succeeds (returns a usable value, never
        // `usize::MAX`) — but we defend against that documented escape
        // hatch anyway with a manual fallback, rather than silently
        // mis-aligning memory if a future compiler ever behaves
        // differently here.
        let pad = current.align_offset(layout.align());
        let pad = if pad == usize::MAX {
            let addr = current as usize;
            let align = layout.align();
            (align - (addr % align)) % align
        } else {
            pad
        };

        let start = cur.checked_add(pad).expect("Arena offset overflow");
        let end = start
            .checked_add(layout.size())
            .expect("Arena allocation size overflow");
        assert!(
            end <= self.capacity,
            "Arena out of memory: need {} bytes ({} padding + {} payload), \
             only {} of {} bytes left",
            pad + layout.size(),
            pad,
            layout.size(),
            self.capacity.saturating_sub(cur),
            self.capacity
        );

        // SAFETY:
        // - `end <= self.capacity` (just asserted) and `start <= end`, so
        //   `current.add(pad)` through `current.add(pad) + size_of::<T>()`
        //   stays entirely within the arena's single backing allocation.
        // - `current.add(pad)` is aligned for `T` by construction of `pad`
        //   above, satisfying `write`'s alignment requirement.
        // - This offset range has never been written as a live `T` that is
        //   still readable through a reference (the bump cursor only moves
        //   forward between resets, so `alloc` never returns two
        //   overlapping regions before a `reset`), so writing an
        //   uninitialized `T` here cannot run a stale destructor or alias
        //   an existing `&T`.
        // - The `&T` we return borrows `self`, so by the struct's design
        //   (documented above) it cannot outlive the arena, and the borrow
        //   checker prevents `reset`/`Drop` while it's alive.
        unsafe {
            let dst = current.add(pad) as *mut T;
            dst.write(val);
            self.offset.set(end);
            &*dst
        }
    }

    /// Reclaims all space allocated so far by rewinding the bump cursor to
    /// zero. Requires `&mut self` — see the struct-level docs for why that,
    /// combined with `alloc`'s `&self -> &T` signature, is what makes this
    /// safe: the borrow checker refuses to compile a call to `reset` while
    /// any reference previously returned by `alloc` is still alive.
    ///
    /// Does not run `Drop` for previously-allocated values — see the
    /// struct-level docs.
    pub fn reset(&mut self) {
        self.offset.set(0);
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        // SAFETY: `self.buf` was allocated by `alloc(self.layout)` in
        // `Arena::new` and has not been freed anywhere else (the arena
        // owns it exclusively, and `Arena` is not `Clone`/`Copy`).
        // `self.layout` is exactly the layout that allocation was made
        // with, matching `dealloc`'s contract. As documented on the
        // struct, this does not run `Drop` for values written into the
        // arena.
        unsafe { dealloc(self.buf.as_ptr(), self.layout) };
    }
}

/// A parsed log record: four metrics from one row of raw input. `Copy`
/// because the arena that stores these does not run `Drop` on reset (see
/// [`Arena`]'s docs) — that's a safe thing to do precisely because this
/// type owns no external resources.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogRecord {
    pub latency: f32,
    pub cpu: f32,
    pub mem: f32,
    pub err_rate: f32,
}

/// Parses a flat batch of raw metrics (groups of 4: `latency, cpu, mem,
/// err_rate`) into [`LogRecord`]s bump-allocated out of `arena`.
///
/// The naive way to write this is to heap-allocate each `LogRecord`
/// on-the-fly (e.g. `Box::new` per record, or a `Vec<LogRecord>` that grows
/// and reallocates as it's filled) once per *batch*, over and over as
/// batches stream in. Allocating out of an [`Arena`] instead turns that
/// repeated per-batch allocation churn into a handful of bump pointer
/// bumps, with the whole batch's worth of memory reclaimed in one O(1)
/// [`Arena::reset`] call once the batch has been processed — no
/// per-record `free`/`drop` bookkeeping needed.
///
/// Any incomplete trailing group of fewer than 4 floats is ignored.
pub fn parse_batch<'a>(arena: &'a Arena, raw: &[f32]) -> Vec<&'a LogRecord> {
    let (chunks, _incomplete_trailing_group) = raw.as_chunks::<4>();
    chunks
        .iter()
        .map(|&[latency, cpu, mem, err_rate]| {
            arena.alloc(LogRecord {
                latency,
                cpu,
                mem,
                err_rate,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tiny hand-rolled linear congruential generator so tests have
    /// deterministic, seeded "random" input without pulling in a `rand`
    /// dependency. Constants are the well-known ones from Knuth's MMIX.
    struct Lcg(u64);

    impl Lcg {
        fn new(seed: u64) -> Self {
            Lcg(seed)
        }

        fn next_u32(&mut self) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 33) as u32
        }

        /// Uniform-ish f32 in `[lo, hi)`.
        fn next_f32(&mut self, lo: f32, hi: f32) -> f32 {
            let t = self.next_u32() as f32 / u32::MAX as f32;
            lo + t * (hi - lo)
        }
    }

    fn assert_close(scalar: f32, simd: f32, len: usize) {
        let diff = (scalar - simd).abs();
        let scale = scalar.abs().max(simd.abs()).max(1e-6);
        let rel = diff / scale;
        assert!(
            rel <= 1e-4,
            "len={len}: scalar={scalar} simd={simd} diff={diff} rel={rel} (tolerance 1e-4)"
        );
    }

    #[test]
    fn test_simd_matches_scalar_epsilon() {
        // Awkward lengths deliberately straddle the 8-lane SIMD chunk size:
        // 0 (empty), 1 and 7 (all tail, no full chunk), 8 (exactly one
        // chunk, no tail), 9 (one chunk + 1 tail element), and a large
        // length with a nonzero remainder to exercise the tail loop at
        // scale.
        let lengths = [0usize, 1, 7, 8, 9, 1_000_003];

        for &len in &lengths {
            let mut rng = Lcg::new(0xDEAD_BEEF ^ len as u64);
            // Deliberately range past [0, 100] on both ends so the clamp
            // branch in both implementations is exercised, not just the
            // "normal" middle of the range.
            let data: Vec<f32> = (0..len).map(|_| rng.next_f32(-50.0, 150.0)).collect();

            let scalar = score_scalar(&data);
            let simd = score_simd(&data);
            assert_close(scalar, simd, len);
        }
    }

    #[test]
    fn test_arena_alignment() {
        let arena = Arena::new(4096);

        #[repr(align(32))]
        #[derive(Debug, PartialEq)]
        struct Over32([u8; 4]);

        // Interleave odd-sized/low-alignment allocations with
        // higher-alignment ones so the arena is forced to insert padding
        // between them, not just get lucky with a naturally-aligned
        // starting offset.
        let a = arena.alloc(1u8);
        let b = arena.alloc(2u16);
        let c = arena.alloc(3u8);
        let d = arena.alloc(4u64);
        let e = arena.alloc(5u8);
        let f = arena.alloc(6u128);
        let g = arena.alloc(Over32([7, 8, 9, 10]));

        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
        assert_eq!(*d, 4);
        assert_eq!(*e, 5);
        assert_eq!(*f, 6);
        assert_eq!(*g, Over32([7, 8, 9, 10]));

        assert_eq!((a as *const u8 as usize) % align_of::<u8>(), 0);
        assert_eq!((b as *const u16 as usize) % align_of::<u16>(), 0);
        assert_eq!((c as *const u8 as usize) % align_of::<u8>(), 0);
        assert_eq!((d as *const u64 as usize) % align_of::<u64>(), 0);
        assert_eq!((e as *const u8 as usize) % align_of::<u8>(), 0);
        assert_eq!((f as *const u128 as usize) % align_of::<u128>(), 0);
        assert_eq!((g as *const Over32 as usize) % 32, 0);
    }

    #[test]
    fn test_arena_reset_reuses() {
        // Exactly enough room for one `i64` and not a byte more, so if
        // `reset` failed to actually reclaim space, the *second* iteration
        // of this loop would panic with "Arena out of memory".
        let mut arena = Arena::new(std::mem::size_of::<i64>());

        let mut previous_addr: Option<usize> = None;
        for i in 0..1000i64 {
            let v = arena.alloc(i);
            assert_eq!(*v, i);

            let addr = v as *const i64 as usize;
            if let Some(prev) = previous_addr {
                assert_eq!(
                    addr, prev,
                    "reset should rewind the bump cursor so this slot is reused"
                );
            }
            previous_addr = Some(addr);

            arena.reset();
        }
    }

    #[test]
    fn test_parse_batch_uses_arena() {
        let arena = Arena::new(4096);
        let raw = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, /* trailing partial group, ignored */ 9.0, 10.0];

        let records = parse_batch(&arena, &raw);

        assert_eq!(records.len(), 2);
        assert_eq!(
            *records[0],
            LogRecord {
                latency: 1.0,
                cpu: 2.0,
                mem: 3.0,
                err_rate: 4.0
            }
        );
        assert_eq!(
            *records[1],
            LogRecord {
                latency: 5.0,
                cpu: 6.0,
                mem: 7.0,
                err_rate: 8.0
            }
        );
    }

    #[test]
    fn test_score_scalar_empty_and_small() {
        assert_eq!(score_scalar(&[]), 0.0);
        // Single element uses WEIGHTS[0].
        let v = score_scalar(&[50.0]);
        assert!((v - 0.5 * WEIGHTS[0]).abs() < 1e-6);
    }
}
