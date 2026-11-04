//! # Lab 1 — Memory Layout (SOLUTION)
//!
//! Rust does not promise you a struct's field order in memory matches its
//! declaration order. That freedom is what lets the compiler pack fields
//! tightly by default. This crate demonstrates the three representations
//! you actually meet in production code and shows how each one trades
//! *predictability* for *automatic optimization*:
//!
//! - **`repr(Rust)`** (the default, no attribute at all): the compiler is
//!   free to reorder fields and pick the smallest possible discriminant
//!   for enums. You get the best size for free, but the layout is
//!   unspecified — it can differ between compiler versions and you must
//!   never assume a byte offset or hand it across an FFI boundary.
//! - **`repr(C)`**: fields are laid out in the *exact order you declared
//!   them*, using the same rules a C compiler would (align each field to
//!   its own alignment, pad the whole struct up to its alignment). This
//!   is what you reach for whenever the byte layout must be predictable —
//!   FFI structs, on-the-wire protocol headers, anything read by another
//!   language or another build of your own program. The cost: you are now
//!   responsible for field order. The compiler will no longer save you
//!   from padding waste.
//! - **`repr(transparent)`**: only legal on a single-field newtype (plus
//!   possibly ZST fields). It guarantees the wrapper has *exactly* the
//!   same size, alignment and ABI as its one non-ZST field, so it can be
//!   passed anywhere the inner type is expected (e.g. across FFI, or via
//!   `transmute`) — useful for typed newtypes (`PayloadLen(u32)`) that
//!   should cost nothing over the raw value.
//!
//! Padding, in one sentence: every field must start at an offset that is a
//! multiple of its own alignment, and the whole struct's size must be a
//! multiple of its largest field's alignment (so it aligns correctly
//! inside an array) — the compiler inserts unnamed padding bytes wherever
//! that forces a gap.

use std::mem::size_of;

/* -------------------------------------------------------------------- */
/*  PacketHeader — repr(C), padding-optimized                            */
/* -------------------------------------------------------------------- */

/// A wire-format telemetry packet header.
///
/// This is `repr(C)` because it is serialized as-is onto a network wire —
/// its byte layout is a contract with whatever reads the other end, so it
/// cannot be left to the compiler's (unspecified, `repr(Rust)`) discretion.
///
/// Because `repr(C)` lays fields out in declaration order, *we* now do the
/// job the compiler would otherwise do automatically: fields are declared
/// **largest alignment first**, so each one lands on a boundary the
/// previous field already satisfied, leaving no gaps until the final
/// implicit tail-padding byte:
///
/// ```text
/// offset  0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
///        +-------------------------------+---------------+-------+---+---+
///        |        timestamp: u64         |  payload_len  | kind  |fl.|pad|
///        +-------------------------------+---------------+-------+---+---+
/// ```
/// (columns line up with the `offset` ruler above: each byte is 4 columns
/// wide, so `timestamp`'s box spans bytes 0–7, `payload_len` bytes 8–11,
/// `kind` bytes 12–13, `fl.` (flag) byte 14, and `pad` the tail-padding
/// byte 15 — `fl.`/`pad` are abbreviated only because a single byte is too
/// narrow to spell out the field name.)
///
/// `size_of::<PacketHeader>() == 16`: 8 (timestamp) + 4 (payload_len) +
/// 2 (kind) + 1 (flag) = 15 bytes of real data, rounded up to 16 (a
/// multiple of the struct's 8-byte alignment, inherited from `u64`) —
/// exactly one byte of unavoidable tail padding. Compare that to the
/// "natural" (flag, timestamp, kind, payload_len) order: it needs 9
/// padding bytes (7 before `timestamp` so it lands on an 8-byte boundary,
/// plus 2 before `payload_len` so it lands on a 4-byte boundary) and pads
/// out to a 24-byte struct. These are two different measures, easy to
/// conflate: the *padding-byte count* drops from 9 to 1, while the
/// *total size* drops from 24 to 16 — an 8-byte size delta that is not the
/// same number as the padding-byte count (9 vs. 1). See
/// `test_repr_c_layout` for the exact per-field offsets and the starter
/// crate for the wasteful version.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    pub timestamp: u64,
    pub payload_len: u32,
    pub kind: u16,
    pub flag: u8,
}

/// The byte budget `PacketHeader` is designed to fit: one 8-byte alignment
/// unit, with only the single unavoidable tail-padding byte wasted.
pub const TARGET_HEADER_SIZE: usize = 16;

// Compile-time proof that stays true across recompiles: if a future edit
// reintroduces padding, the crate fails to *build*, not just to test.
const _: () = assert!(size_of::<PacketHeader>() == TARGET_HEADER_SIZE);

/* -------------------------------------------------------------------- */
/*  PayloadLen — repr(transparent)                                       */
/* -------------------------------------------------------------------- */

/// A typed newtype over a raw `u32` payload length.
///
/// `repr(transparent)` guarantees this has the *identical* size, alignment
/// and ABI as the `u32` it wraps — there is no hidden tag, no padding, and
/// it is legal to pass this across an `extern "C"` boundary anywhere a
/// `u32` is expected, or `transmute` between the two. Compare this to a
/// plain (no-repr) single-field struct: that one *also* happens to have
/// the same size today (repr(Rust) has no reason to pad a one-field
/// struct), but nothing guarantees it will stay that way — `transparent`
/// is the difference between "currently true" and "load-bearing".
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PayloadLen(pub u32);

/* -------------------------------------------------------------------- */
/*  Message — repr(Rust), mixed-size variants                            */
/* -------------------------------------------------------------------- */

/// A telemetry message with mixed-size variants.
///
/// An enum's in-memory footprint must be large enough to hold *any* of its
/// variants, so a single oversized variant inflates every value of the
/// type — even the ones carrying no payload at all. `Data` boxes its
/// payload for exactly this reason: putting the 64-byte array behind a
/// pointer means every variant's inline footprint is now governed by the
/// largest *remaining* inline field (`Error`'s `String`, at 24 bytes)
/// instead of the 64-byte array. See `test_message_size_after_boxing` for
/// the before/after numbers, and `test_enum_niche` for why boxing a
/// variant's payload can shrink an enum by more than just "the array is
/// gone".
///
/// **Guaranteed vs. observed**: the 72→24 shrink rides on `String`'s
/// internal capacity field happening to use a niche type today. That is
/// *current `rustc` behavior* you can check with `-Z print-type-sizes`
/// (see the README's stretch goals) — the standard library does not
/// document this as a promised layout, unlike the `Box`/`NonZero*`
/// guarantees in `test_enum_niche`/`test_nonzero_niche`. Do not build an
/// FFI or cross-version contract on top of it.
///
/// This type deliberately has *no* `#[repr(..)]` attribute: leaving it
/// `repr(Rust)` is what allows the niche-filling optimization described
/// above. `repr(C)` would force an explicit, separate discriminant field
/// instead (a predictable C-style tagged union) — the right choice for an
/// enum crossing an FFI boundary, but not needed for an in-process type
/// like this one, and it would give up the packing win for nothing.
#[derive(Debug, Clone)]
pub enum Message {
    /// No payload at all.
    Ping,
    /// A single-byte acknowledgement code.
    Ack(u8),
    /// A large, fixed-size payload — boxed so it doesn't dominate the
    /// enum's size (see the type-level doc comment).
    Data(Box<[u8; 64]>),
    /// A human-readable error message.
    Error(String),
}

/// The target size for `Message` after boxing the oversized `Data`
/// variant. It happens to equal `size_of::<String>()`: the compiler folds
/// the discriminant into unused capacity inside `String`'s internal
/// fields rather than adding a separate tag byte (see
/// `test_message_size_after_boxing`'s doc comment for the mechanism).
pub const TARGET_MESSAGE_SIZE: usize = size_of::<String>();

const _: () = assert!(size_of::<Message>() == TARGET_MESSAGE_SIZE);

/// Same variant set as [`Message`], but with the payload stored inline
/// instead of boxed. It exists purely so tests can show the size
/// difference boxing makes; nothing about it is "wrong" Rust, it is just
/// the naive version you'd write before profiling or before running
/// `clippy::large_enum_variant`.
#[cfg(test)]
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum MessageUnboxed {
    Ping,
    Ack(u8),
    Data([u8; 64]),
    Error(String),
}

/* -------------------------------------------------------------------- */
/*  LinkStatus — repr(u8), fieldless niche-optimization demo             */
/* -------------------------------------------------------------------- */

/// A fixed-width, FFI-friendly status code.
///
/// `#[repr(u8)]` here is about *ABI stability*, not size: this enum has no
/// payload, so it is already exactly one byte under the default
/// `repr(Rust)` too. What `repr(u8)` buys you is a *guarantee* that the
/// discriminant is stored as a `u8` with these exact values — safe to
/// hand to C, encode on the wire, or `as u8` cast.
///
/// Because only 4 of the 256 possible byte values are valid `LinkStatus`
/// discriminants, `Option<LinkStatus>` needs no extra byte for its own
/// tag: the compiler represents `None` using one of the 252 otherwise-
/// unused byte patterns (this is the same "niche-filling" mechanism that
/// makes `Option<bool>` one byte instead of two). See
/// `test_repr_u8_fieldless_niche`.
///
/// **Guaranteed vs. observed**: unlike `Box`/`NonZero*` (see
/// `test_enum_niche`, `test_nonzero_niche`), the standard library makes no
/// documented promise that a user-defined `#[repr(u8)]` enum like this one
/// gets niche-filled — this is *current `rustc` behavior*, verifiable with
/// `-Z print-type-sizes`, not a cross-version or FFI contract. Depend on it
/// for local size intuition; re-verify after a toolchain bump if it matters
/// to you.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStatus {
    Idle = 0,
    Connecting = 1,
    Connected = 2,
    Closing = 3,
}

/// **Guaranteed vs. observed niche optimization**: the tests below split
/// into two different epistemic categories — conflating them is a common
/// bug magnet when you later depend on a size fact in a public interface.
///
/// - `test_enum_niche` (`Option<Box<u32>>`) and `test_nonzero_niche`
///   (`Option<NonZeroU8>`) rely on a **documented standard-library
///   guarantee**: std's `Option` docs ("Representation" section) name an
///   explicit, promised set of types eligible for null-pointer/niche
///   optimization — `Box<T>`, `&T`/`&mut T`, `fn` pointers, `NonNull<T>`,
///   the `NonZero*` types, and `#[repr(transparent)]` wrappers around any
///   of those. Safe to depend on across compiler versions and in FFI-facing
///   size assumptions.
/// - `test_repr_u8_fieldless_niche` (`Option<LinkStatus>`, a *user-defined*
///   `#[repr(u8)]` enum) and `test_message_size_after_boxing` (the boxed
///   `Message`'s 72→24 shrink, via `String`'s internal capacity niche) are
///   **current `rustc` behavior**, not a documented contract — nothing in
///   the reference names user enums or `String`'s internals as guaranteed
///   niche donors. Verifiable with `-Z print-type-sizes` (README stretch
///   goals), but not something a future compiler is obligated to preserve.
///
/// Rule of thumb: depend on documented guarantees in interfaces; re-verify
/// observed layout whenever you upgrade the toolchain.
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, offset_of};
    use std::num::NonZeroU8;

    /// RED before Step 1 of the starter: with the "natural" declaration
    /// order (flag, timestamp, kind, payload_len), `repr(C)` produces a
    /// 24-byte struct with 9 padding bytes (7 before `timestamp`, 2 before
    /// `payload_len`; 24 total − 15 data). GREEN once the fields are
    /// reordered largest-alignment-first, as they are in this crate: a
    /// 16-byte struct with 1 tail-padding byte — a size delta of 8 bytes,
    /// which is not the same number as the padding-byte-count delta (9
    /// vs. 1). See the type-level doc comment above for the full
    /// breakdown of both measures.
    #[test]
    fn test_optimized_header_size() {
        assert_eq!(size_of::<PacketHeader>(), 16);
        assert_eq!(size_of::<PacketHeader>(), TARGET_HEADER_SIZE);
    }

    /// `offset_of!` reads back the *actual* compiled layout — this only
    /// gives stable, meaningful numbers because `PacketHeader` is
    /// `repr(C)`. Doing the same on a `repr(Rust)` type would still
    /// compile, but the numbers would be an implementation detail you
    /// have no right to depend on.
    #[test]
    fn test_repr_c_layout() {
        assert_eq!(offset_of!(PacketHeader, timestamp), 0);
        assert_eq!(offset_of!(PacketHeader, payload_len), 8);
        assert_eq!(offset_of!(PacketHeader, kind), 12);
        assert_eq!(offset_of!(PacketHeader, flag), 14);
        // 14 (last field starts) + 1 (its own size) = 15 bytes of data,
        // rounded up to the struct's 8-byte alignment.
        assert_eq!(size_of::<PacketHeader>(), 16);
    }

    /// `align_of` matters independently of `size_of`: two types can be
    /// the same size but need different alignment. `PacketHeader`'s
    /// alignment is 8 because its strictest field (`timestamp: u64`) is
    /// 8-byte aligned — the struct as a whole must satisfy that too, or
    /// an array of headers would misalign every element after the first.
    #[test]
    fn test_header_alignment() {
        assert_eq!(align_of::<PacketHeader>(), align_of::<u64>());
        assert_eq!(align_of::<PacketHeader>(), 8);
    }

    /// `repr(transparent)` is not just "happens to be the same size" —
    /// it is a *guarantee*, checked here for both size and alignment.
    #[test]
    fn test_repr_transparent_matches_inner() {
        assert_eq!(size_of::<PayloadLen>(), size_of::<u32>());
        assert_eq!(align_of::<PayloadLen>(), align_of::<u32>());
    }

    /// The headline niche-optimization example: `Box<T>` is backed by a
    /// `NonNull<T>` pointer, which can never legally be the all-zero bit
    /// pattern. `Option<Box<T>>` reuses that one "impossible" bit pattern
    /// to represent `None`, so wrapping a `Box` in `Option` costs nothing
    /// — no separate tag, no size increase over the `Box` alone. This is a
    /// **documented std guarantee** (see the tests module doc comment
    /// above), safe to depend on in a public interface.
    #[test]
    fn test_enum_niche() {
        assert_eq!(size_of::<Box<u32>>(), 8);
        assert_eq!(size_of::<Option<Box<u32>>>(), size_of::<Box<u32>>());
        assert_eq!(size_of::<Option<Box<u32>>>(), 8);
    }

    /// The standard library ships this same trick pre-packaged:
    /// `NonZeroU8`'s only invalid bit pattern is 0, so `Option<NonZeroU8>`
    /// uses 0 for `None` and needs no extra byte — same size as a plain
    /// `u8`, unlike `Option<u8>` which must grow to 2 bytes because every
    /// `u8` bit pattern is already a valid value. Like `Box`, `NonZero*` is
    /// a **documented std guarantee** (see the tests module doc comment
    /// above), not merely an observed fact.
    #[test]
    fn test_nonzero_niche() {
        assert_eq!(size_of::<NonZeroU8>(), 1);
        assert_eq!(size_of::<Option<NonZeroU8>>(), 1);
        assert_eq!(size_of::<Option<u8>>(), 2, "u8 has no spare bit pattern to steal");
    }

    /// A fieldless `#[repr(u8)]` enum has exactly the same niche-filling
    /// eligibility as `NonZeroU8`: only 4 of 256 byte values are valid, so
    /// `Option<LinkStatus>` reuses one of the other 252 for `None`. Unlike
    /// `NonZeroU8`, this is **observed current-`rustc` behavior, not a
    /// documented guarantee** for user-defined enums (see the tests module
    /// doc comment above).
    #[test]
    fn test_repr_u8_fieldless_niche() {
        assert_eq!(size_of::<LinkStatus>(), 1);
        assert_eq!(size_of::<Option<LinkStatus>>(), 1);
    }

    /// Boxing `Data`'s payload shrinks `Message` from 72 bytes down to
    /// exactly `size_of::<String>()` (24 bytes) — not 72 - 64 + 8 = 16, and
    /// not 24 + 8 for a separate tag either. `String`'s internal capacity
    /// field uses a standard-library niche type reserving its top bit
    /// (an allocation can never be `isize::MAX + 1` bytes), which hands
    /// the compiler more than enough spare bit patterns to encode all
    /// three of `Message`'s other variants without a dedicated
    /// discriminant byte. This is the same *kind* of trick as
    /// `test_enum_niche` and `test_nonzero_niche`, just with a much
    /// roomier niche donor — and, unlike those two, it is **observed
    /// current-`rustc` behavior, not a documented guarantee** (see the
    /// tests module doc comment above). (Curious how the compiler picked
    /// this apart? See the README's stretch goals for
    /// `-Z print-type-sizes`.)
    #[test]
    fn test_message_size_after_boxing() {
        assert_eq!(size_of::<MessageUnboxed>(), 72, "unboxed: dominated by the 64-byte array");
        assert_eq!(size_of::<Message>(), 24, "boxed: dominated by String, tag folded into its niche");
        assert_eq!(size_of::<Message>(), TARGET_MESSAGE_SIZE);
        assert!(size_of::<Message>() < size_of::<MessageUnboxed>());
    }
}
