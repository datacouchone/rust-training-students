//! Lab 4 — typestate pattern + const generics + GATs.
//!
//! Three building blocks live in this crate:
//!
//! 1. [`Socket`] — a typestate-encoded socket. The current state
//!    (`Closed`, `Bound`, `Connected`) is carried in the type parameter, so
//!    illegal operations (e.g. calling `send` before `connect`) are
//!    rejected by the compiler because the method simply does not exist on
//!    that instantiation of `Socket<State>`. No real networking happens —
//!    "sending" appends to an in-memory buffer.
//!
//!    The state markers, the sealed `SocketState` trait, and `Socket::new`
//!    are already written. **Your job**: implement `bind`, `connect`, and
//!    `send` (marked `todo!()` below).
//!
//! 2. [`RingBuffer`] — a fixed-capacity ring buffer whose capacity `N` is a
//!    `const` generic parameter, fully implemented already. Read it: it's
//!    your reference for how `const N: usize` flows through a type and its
//!    `impl` block.
//!
//! 3. [`LendingIterator`] — a GAT-based iterator trait whose `Item<'a>`
//!    borrows from the iterator itself. The trait and the `WindowsMut`
//!    struct are given; **your job**: implement `WindowsMut`'s `next`
//!    (marked `todo!()` below) so it hands out overlapping *mutable*
//!    windows over the underlying buffer.

use std::marker::PhantomData;

// ===========================================================================
// 1. Typestate socket
// ===========================================================================

mod sealed {
    // Sealed supertrait: only types in this module may implement
    // `SocketState`, so downstream crates can't invent bogus states.
    pub trait Sealed {}
    impl Sealed for super::Closed {}
    impl Sealed for super::Bound {}
    impl Sealed for super::Connected {}
}

/// Marker trait implemented only by the socket's typestate markers
/// (`Closed`, `Bound`, `Connected`). Sealed so no other type can be used as
/// `Socket<State>`'s state parameter.
pub trait SocketState: sealed::Sealed {}

/// The socket has not been bound to a local address yet.
pub struct Closed;
/// The socket is bound to a local address but not yet connected.
pub struct Bound;
/// The socket is connected and may send data.
pub struct Connected;

impl SocketState for Closed {}
impl SocketState for Bound {}
impl SocketState for Connected {}

/// A typestate-encoded, in-memory network socket. `State` is one of the
/// zero-sized markers above; it exists purely at compile time (via
/// [`PhantomData`]) to select which methods are available.
///
/// There is no real I/O here — `send` appends to an internal `Vec<u8>` so
/// the lesson stays on the type-level state machine, not sockets.
pub struct Socket<State: SocketState> {
    local_addr: Option<String>,
    sent: Vec<u8>,
    _state: PhantomData<State>,
}

impl Socket<Closed> {
    /// Entry point: a fresh, unbound socket. Already implemented — start
    /// reading here.
    pub fn new() -> Self {
        Socket {
            local_addr: None,
            sent: Vec::new(),
            _state: PhantomData,
        }
    }

    /// `Closed -> Bound`: record the local address and move on. Consumes
    /// `self` so the old `Socket<Closed>` can't be reused after binding.
    ///
    /// TODO: store `addr` (converted to `String`) as the new socket's
    /// `local_addr`, carry `self.sent` forward unchanged, and return the
    /// resulting `Socket<Bound>`.
    pub fn bind(self, addr: impl Into<String>) -> Socket<Bound> {
        todo!("Closed -> Bound: store addr, carry the rest of the fields forward")
    }
}

impl Default for Socket<Closed> {
    fn default() -> Self {
        Self::new()
    }
}

impl Socket<Bound> {
    /// `Bound -> Connected`: no new data is needed, the socket simply
    /// becomes eligible to send.
    ///
    /// TODO: carry every field forward into a `Socket<Connected>` — this
    /// transition doesn't need any new information, only a new type.
    pub fn connect(self) -> Socket<Connected> {
        todo!("Bound -> Connected: carry every field forward, nothing to compute")
    }
}

impl Socket<Connected> {
    /// Send bytes on a connected socket, returning the number of bytes
    /// "sent" (i.e. appended to the internal buffer).
    ///
    /// TODO: append `data` to `self.sent` and return `data.len()`.
    pub fn send(&mut self, data: &[u8]) -> usize {
        todo!("append data to self.sent, return the number of bytes appended")
    }

    /// Every byte ever sent on this socket, in order. Test-only window into
    /// the modeled "wire".
    pub fn bytes_sent(&self) -> &[u8] {
        &self.sent
    }
}

impl<State: SocketState> Socket<State> {
    /// The bound local address, if any. Available in every state so you
    /// can check it survived a transition.
    pub fn local_addr(&self) -> Option<&str> {
        self.local_addr.as_deref()
    }
}

// ===========================================================================
// 2. Const-generic ring buffer (fully implemented — read for reference)
// ===========================================================================

/// A fixed-capacity ring buffer. Capacity `N` is a `const` generic, so it is
/// part of the type — no heap allocation and no runtime capacity field.
/// Pushing past capacity overwrites (and returns) the oldest element.
pub struct RingBuffer<T, const N: usize> {
    slots: [Option<T>; N],
    /// Index the *next* `push` will write to.
    head: usize,
    /// Number of live elements, saturating at `N`.
    len: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    // Evaluated once per monomorphization: turns `RingBuffer::<T, 0>` into a
    // compile-time error the first time it's actually used, since a
    // capacity of zero makes `% N` undefined.
    const CHECK_CAPACITY: () = assert!(N > 0, "RingBuffer capacity N must be greater than 0");

    /// Create an empty ring buffer. Capacity is fixed at `N` by the type.
    pub fn new() -> Self {
        let () = Self::CHECK_CAPACITY;
        RingBuffer {
            slots: std::array::from_fn(|_| None),
            head: 0,
            len: 0,
        }
    }

    /// Compile-time capacity of this buffer.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Number of elements currently stored (`0..=N`).
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == N
    }

    /// Push a value. If the buffer is full, the oldest element is
    /// overwritten and returned; otherwise returns `None`.
    pub fn push(&mut self, value: T) -> Option<T> {
        let idx = self.head;
        let evicted = self.slots[idx].replace(value);
        self.head = (self.head + 1) % N;
        if self.len < N {
            self.len += 1;
            None
        } else {
            evicted
        }
    }

    /// Iterate stored elements oldest-to-newest.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let start = if self.len < N { 0 } else { self.head };
        (0..self.len).map(move |i| self.slots[(start + i) % N].as_ref().unwrap())
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// 3. GAT lending iterator + mutable sliding windows
// ===========================================================================

/// An iterator whose `Item` may borrow from the iterator itself, expressed
/// with a generic associated type. `std::iter::Iterator::Item` has no
/// lifetime parameter, so it cannot express "this item borrows from
/// `&mut self`" — that's exactly what GATs add.
///
/// **HRTB.** A function generic over some `I: LendingIterator` can't
/// name one lifetime for `Item` — each `next()` call's borrow is scoped
/// to that call alone. It needs a *higher-ranked* bound instead (a
/// self-contained stand-in trait below, since a doctest can't presume a
/// crate name):
///
/// ```
/// trait Lend { type Item<'a> where Self: 'a; fn next(&mut self) -> Option<Self::Item<'_>>; }
/// fn sum_first<I: Lend>(it: &mut I) -> u32
/// where
///     for<'a> I::Item<'a>: AsRef<[u32]>,
/// {
///     let mut t = 0;
///     while let Some(x) = it.next() { t += x.as_ref().first().copied().unwrap_or(0); }
///     t
/// }
/// ```
///
/// This signature compiles, but calling it on a genuinely-borrowing type
/// like `WindowsMut<'_, u32>` currently doesn't: rustc says the `for<'a>`
/// bound "implies a `'static` lifetime" — a known limitation of
/// higher-ranked bounds on GAT projections. That's why `next` itself uses
/// the *elided*, per-call `Item<'_>` rather than a where-bound HRTB.
///
/// **Variance.** `&'a mut [T]` (this impl's `Item`) is *invariant* in
/// `T`, unlike covariant `&'a T`: you can't treat a `&mut Vec<&'static
/// str>` as a `&mut Vec<&'a str>` — someone could write a short-lived
/// reference through it that the original owner later reads as
/// `'static`. (The lifetime `'a` itself is still covariant —
/// reborrowing shorter is always fine.) Same discipline behind `where
/// Self: 'a`: the borrow `next(&mut self)` returns can't outlive
/// `Self` (`WindowsMut`'s `'buf`), so the GAT requires `Self: 'a` —
/// automatic for `next`'s elided lifetime, not guaranteed for an
/// arbitrary named one.
pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

/// Yields overlapping **mutable** windows of length `window_size` over a
/// slice, sliding forward by one element each call. This cannot be built on
/// the ordinary `Iterator` trait: consecutive windows alias the same
/// memory, so each yielded item's lifetime must be tied to the `&mut self`
/// borrow of that particular `next()` call (a lending iterator), not to the
/// iterator's own lifetime parameter (which is what `Iterator::Item` would
/// require).
pub struct WindowsMut<'buf, T> {
    buf: &'buf mut [T],
    window_size: usize,
    pos: usize,
}

impl<'buf, T> WindowsMut<'buf, T> {
    pub fn new(buf: &'buf mut [T], window_size: usize) -> Self {
        assert!(window_size > 0, "window_size must be non-zero");
        WindowsMut {
            buf,
            window_size,
            pos: 0,
        }
    }
}

impl<'buf, T> LendingIterator for WindowsMut<'buf, T> {
    type Item<'a>
        = &'a mut [T]
    where
        Self: 'a;

    /// TODO: return `Some(window)` where `window` is a `&mut [T]` of length
    /// `self.window_size` starting at `self.pos`, then advance `self.pos`
    /// by 1. Return `None` once a full window no longer fits.
    ///
    /// Hint: you can't get this from safe slicing alone (e.g.
    /// `split_at_mut`) because consecutive windows overlap — you'll need
    /// `self.buf.as_mut_ptr()` and `std::slice::from_raw_parts_mut` inside
    /// an `unsafe` block. Think about why it's still sound: only one
    /// window is ever alive at a time, because each one borrows `self`
    /// mutably for exactly as long as the caller holds it.
    fn next(&mut self) -> Option<Self::Item<'_>> {
        todo!("slide a &mut [T] window of length self.window_size across self.buf")
    }
}
