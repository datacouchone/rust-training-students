//! # Lab 1 — Memory Layout (STARTER)
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
//!   from padding waste — that is your job in this lab.
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
//!
//! ## Your job in this lab
//!
//! `PacketHeader` and `Message` below are laid out the way an engineer
//! would write them without thinking about memory layout at all — "just
//! declare the fields in whatever order makes sense to read". Follow the
//! `TODO(Step N)` comments to fix both.

use std::mem::size_of;

/* -------------------------------------------------------------------- */
/*  PacketHeader — repr(C), currently padding-wasteful                   */
/* -------------------------------------------------------------------- */

/// A wire-format telemetry packet header.
///
/// This is `repr(C)` because it is serialized as-is onto a network wire —
/// its byte layout is a contract with whatever reads the other end, so it
/// cannot be left to the compiler's (unspecified, `repr(Rust)`) discretion.
///
/// Because `repr(C)` lays fields out in *declaration order*, the fields
/// below are currently declared in the order an engineer would naturally
/// think of them (flag, timestamp, kind, payload_len) — and that costs
/// bytes. Run `cargo test -p lab1-layout-starter -- --ignored --nocapture`
/// after Step 1 to see the improvement for yourself.
///
/// TODO(Step 1): Reorder these four fields so the struct packs into 16
/// bytes instead of 24. The rule: declare fields from *largest alignment
/// to smallest* (`align_of::<u64>() = 8`, `align_of::<u32>() = 4`,
/// `align_of::<u16>() = 2`, `align_of::<u8>() = 1`). Each field then
/// naturally lands on a boundary the previous field already satisfied, so
/// the only padding left is a single unavoidable tail byte (the struct's
/// size must be a multiple of its 8-byte alignment). Verify with:
/// `println!("{}", std::mem::size_of::<PacketHeader>());` or by running
/// the tests below.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    pub flag: u8,
    pub timestamp: u64,
    pub kind: u16,
    pub payload_len: u32,
}

/// The byte budget `PacketHeader` should fit once you complete Step 1:
/// one 8-byte alignment unit, with only a single unavoidable tail-padding
/// byte wasted.
pub const TARGET_HEADER_SIZE: usize = 16;

// TODO(Step 1, continued): once PacketHeader is reordered, uncomment this
// compile-time assertion. It turns "did I get the layout right?" into a
// *build* failure instead of a test failure — if the struct still isn't
// `TARGET_HEADER_SIZE` bytes, `cargo build` itself will refuse to finish,
// with a message pointing at this exact line.
//
// const _: () = assert!(size_of::<PacketHeader>() == TARGET_HEADER_SIZE);

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
/// is the difference between "currently true" and "load-bearing". Nothing
/// to fix here — this one is already correct; read it as a worked example.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PayloadLen(pub u32);

/* -------------------------------------------------------------------- */
/*  Message — repr(Rust), currently an oversized mixed-variant enum      */
/* -------------------------------------------------------------------- */

/// A telemetry message with mixed-size variants.
///
/// An enum's in-memory footprint must be large enough to hold *any* of its
/// variants, so a single oversized variant inflates every value of the
/// type — even the ones carrying no payload at all. Right now `Data`
/// stores its 64-byte payload inline, which forces `Message` to be huge
/// (measure it: `std::mem::size_of::<Message>()`) even for a bare `Ping`.
///
/// TODO(Step 2): Box the `Data` variant's payload (`Box<[u8; 64]>` instead
/// of `[u8; 64]`). Putting the array behind a pointer means every
/// variant's inline footprint is now governed by the largest *remaining*
/// inline field (`Error`'s `String`, at 24 bytes) instead of the 64-byte
/// array — `Message` should drop from 72 bytes down to `TARGET_MESSAGE_SIZE`.
///
/// **Guaranteed vs. observed**: the 72→24 shrink you'll see rides on
/// `String`'s internal capacity field happening to use a niche type today.
/// That is *current `rustc` behavior* you can check with
/// `-Z print-type-sizes` (see the README's stretch goals) — the standard
/// library does not document this as a promised layout, unlike the
/// `Box`/`NonZero*` guarantees in `test_enum_niche`/`test_nonzero_niche`.
/// Do not build an FFI or cross-version contract on top of it.
///
/// This type deliberately has *no* `#[repr(..)]` attribute: leaving it
/// `repr(Rust)` is what allows the compiler's niche-filling optimization
/// (see `test_repr_u8_fieldless_niche` and `test_enum_niche` below) to
/// apply. `repr(C)` would force an explicit, separate discriminant field
/// instead — the right choice for an enum crossing an FFI boundary, but
/// not needed for an in-process type like this one.
#[derive(Debug, Clone)]
pub enum Message {
    /// No payload at all.
    Ping,
    /// A single-byte acknowledgement code.
    Ack(u8),
    /// A large, fixed-size payload. TODO(Step 2): box this.
    Data([u8; 64]),
    /// A human-readable error message.
    Error(String),
}

/// The target size for `Message` after completing Step 2. It happens to
/// equal `size_of::<String>()`: once boxed, the compiler can fold the
/// discriminant into unused capacity inside `String`'s internal fields
/// rather than adding a separate tag byte.
pub const TARGET_MESSAGE_SIZE: usize = size_of::<String>();

// TODO(Step 2, continued): uncomment once Data is boxed.
// const _: () = assert!(size_of::<Message>() == TARGET_MESSAGE_SIZE);

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
/// makes `Option<bool>` one byte instead of two). Nothing to fix here —
/// this one is already correct; read it as a worked example alongside
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

    /// Fails until you complete Step 1 (reorder `PacketHeader`'s fields).
    #[test]
    #[ignore = "Step 1 not done yet: PacketHeader is still 24 bytes — reorder its fields (see the TODO on the struct)"]
    fn test_optimized_header_size() {
        assert_eq!(size_of::<PacketHeader>(), 16);
        assert_eq!(size_of::<PacketHeader>(), TARGET_HEADER_SIZE);
    }

    /// `offset_of!` reads back the *actual* compiled layout — this only
    /// gives stable, meaningful numbers because `PacketHeader` is
    /// `repr(C)`. These are the offsets you should see *after* Step 1;
    /// they will not match the current (unoptimized) field order.
    #[test]
    #[ignore = "Step 1 not done yet: these offsets describe the reordered layout"]
    fn test_repr_c_layout() {
        assert_eq!(offset_of!(PacketHeader, timestamp), 0);
        assert_eq!(offset_of!(PacketHeader, payload_len), 8);
        assert_eq!(offset_of!(PacketHeader, kind), 12);
        assert_eq!(offset_of!(PacketHeader, flag), 14);
        assert_eq!(size_of::<PacketHeader>(), 16);
    }

    /// `align_of` matters independently of `size_of`: two types can be
    /// the same size but need different alignment. This holds regardless
    /// of field order, so it already passes.
    #[test]
    fn test_header_alignment() {
        assert_eq!(align_of::<PacketHeader>(), align_of::<u64>());
        assert_eq!(align_of::<PacketHeader>(), 8);
    }

    /// `repr(transparent)` is not just "happens to be the same size" —
    /// it is a *guarantee*, checked here for both size and alignment.
    /// Already correct, no TODO for this one.
    #[test]
    fn test_repr_transparent_matches_inner() {
        assert_eq!(size_of::<PayloadLen>(), size_of::<u32>());
        assert_eq!(align_of::<PayloadLen>(), align_of::<u32>());
    }

    /// The headline niche-optimization example: `Box<T>` is backed by a
    /// `NonNull<T>` pointer, which can never legally be the all-zero bit
    /// pattern. `Option<Box<T>>` reuses that one "impossible" bit pattern
    /// to represent `None`, so wrapping a `Box` in `Option` costs nothing.
    /// This is a standard-library fact, not something you need to fix. It
    /// is also a **documented std guarantee** (see the tests module doc
    /// comment above), safe to depend on in a public interface.
    #[test]
    fn test_enum_niche() {
        assert_eq!(size_of::<Box<u32>>(), 8);
        assert_eq!(size_of::<Option<Box<u32>>>(), size_of::<Box<u32>>());
        assert_eq!(size_of::<Option<Box<u32>>>(), 8);
    }

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

    /// Fails until you complete Step 2 (box `Message::Data`'s payload).
    /// Once it passes, note it is **observed current-`rustc` behavior, not
    /// a documented guarantee** — see the tests module doc comment above
    /// and the TODO on `Message`.
    #[test]
    #[ignore = "Step 2 not done yet: Message is still 72 bytes — box the Data variant's payload (see the TODO on the enum)"]
    fn test_message_size_after_boxing() {
        assert_eq!(size_of::<Message>(), TARGET_MESSAGE_SIZE);
        assert_eq!(size_of::<Message>(), 24);
    }
}
