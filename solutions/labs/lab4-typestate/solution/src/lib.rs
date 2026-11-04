//! Lab 4 — typestate pattern + const generics + GATs.
//!
//! Three independent building blocks live in this crate:
//!
//! 1. [`Socket`] — a typestate-encoded socket. The current state
//!    (`Closed`, `Bound`, `Connected`) is carried in the type parameter, so
//!    illegal operations (e.g. calling `send` before `connect`) are
//!    rejected by the compiler because the method simply does not exist on
//!    that instantiation of `Socket<State>`. No real networking happens —
//!    "sending" appends to an in-memory buffer so tests can observe it.
//! 2. [`RingBuffer`] — a fixed-capacity ring buffer whose capacity `N` is a
//!    `const` generic parameter, so the capacity is known and checked at
//!    compile time (no heap allocation, no runtime capacity field).
//! 3. [`LendingIterator`] — a GAT-based iterator trait whose `Item<'a>`
//!    borrows from the iterator itself, implemented by [`WindowsMut`] to
//!    hand out overlapping *mutable* windows over a single buffer. This is
//!    the classic example that motivates GATs: the standard `Iterator`
//!    trait cannot express "each item borrows from `&mut self`".

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
    /// Entry point: a fresh, unbound socket.
    pub fn new() -> Self {
        Socket {
            local_addr: None,
            sent: Vec::new(),
            _state: PhantomData,
        }
    }

    /// `Closed -> Bound`: record the local address. Consumes `self` so the
    /// old `Socket<Closed>` cannot be reused after binding.
    pub fn bind(self, addr: impl Into<String>) -> Socket<Bound> {
        Socket {
            local_addr: Some(addr.into()),
            sent: self.sent,
            _state: PhantomData,
        }
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
    pub fn connect(self) -> Socket<Connected> {
        Socket {
            local_addr: self.local_addr,
            sent: self.sent,
            _state: PhantomData,
        }
    }
}

impl Socket<Connected> {
    /// Send bytes on a connected socket, returning the number of bytes
    /// "sent" (i.e. appended to the internal buffer).
    pub fn send(&mut self, data: &[u8]) -> usize {
        self.sent.extend_from_slice(data);
        data.len()
    }

    /// Every byte ever sent on this socket, in order. Test-only window into
    /// the modeled "wire".
    pub fn bytes_sent(&self) -> &[u8] {
        &self.sent
    }
}

impl<State: SocketState> Socket<State> {
    /// The bound local address, if any. Available in every state so tests
    /// can check it survived a transition.
    pub fn local_addr(&self) -> Option<&str> {
        self.local_addr.as_deref()
    }
}

// ===========================================================================
// 2. Const-generic ring buffer
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

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.pos + self.window_size > self.buf.len() {
            return None;
        }
        let start = self.pos;
        self.pos += 1;

        // SAFETY: `start + window_size <= self.buf.len()` was just checked,
        // so the returned slice is entirely in bounds. Consecutive windows
        // alias the same underlying storage (window `pos` and `pos + 1`
        // share `window_size - 1` elements), which is why this can't be
        // written with safe slicing (e.g. `split_at_mut`) — but it is sound
        // because only one window is ever live at a time: the returned
        // `&'a mut [T]` borrows `*self` for the lifetime `'a` tied to this
        // `next(&mut self)` call, so the borrow checker forces the caller
        // to drop it before the next call to `next()` can produce another
        // one. No two returned windows are ever accessible simultaneously,
        // so there is no aliasing mutable reference at any given moment.
        let ptr = self.buf.as_mut_ptr();
        unsafe {
            let window_ptr = ptr.add(start);
            Some(std::slice::from_raw_parts_mut(window_ptr, self.window_size))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- typestate socket ---------------------------------------------

    #[test]
    fn test_full_transition_path() {
        let socket = Socket::<Closed>::new();
        assert_eq!(socket.local_addr(), None);

        let socket = socket.bind("127.0.0.1:8080");
        assert_eq!(socket.local_addr(), Some("127.0.0.1:8080"));

        let mut socket = socket.connect();
        assert_eq!(socket.local_addr(), Some("127.0.0.1:8080"));

        let n = socket.send(b"hello");
        assert_eq!(n, 5);
        assert_eq!(socket.bytes_sent(), b"hello");
    }

    #[test]
    fn test_send_on_connected() {
        let mut socket = Socket::<Closed>::new().bind("10.0.0.1:9000").connect();

        assert_eq!(socket.send(b"abc"), 3);
        assert_eq!(socket.send(b"defg"), 4);
        assert_eq!(socket.bytes_sent(), b"abcdefg");
    }

    #[test]
    fn test_state_markers_are_zero_sized() {
        // The typestate markers must carry no runtime data — the state
        // lives entirely at the type level via PhantomData.
        assert_eq!(std::mem::size_of::<Closed>(), 0);
        assert_eq!(std::mem::size_of::<Bound>(), 0);
        assert_eq!(std::mem::size_of::<Connected>(), 0);
    }

    // -- const-generic ring buffer --------------------------------------

    #[test]
    fn test_ring_buffer_wraps() {
        let mut rb: RingBuffer<i32, 3> = RingBuffer::new();
        assert_eq!(rb.capacity(), 3);
        assert!(rb.is_empty());

        assert_eq!(rb.push(1), None);
        assert_eq!(rb.push(2), None);
        assert_eq!(rb.push(3), None);
        assert!(rb.is_full());
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);

        // Buffer is full: the next push must wrap around and evict the
        // oldest element (1).
        assert_eq!(rb.push(4), Some(1));
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);

        // Wrap again to be sure it's not a one-shot fluke.
        assert_eq!(rb.push(5), Some(2));
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![3, 4, 5]);
    }

    // -- GAT lending iterator --------------------------------------------

    #[test]
    fn test_lending_windows_mut() {
        let mut data = [0i32, 1, 2, 3, 4];
        let mut windows = WindowsMut::new(&mut data, 3);

        let mut seen = Vec::new();
        while let Some(w) = windows.next() {
            seen.push(w.to_vec());
            for x in w.iter_mut() {
                *x += 100;
            }
        }

        // 5 elements, window length 3 => 3 sliding windows.
        assert_eq!(seen.len(), 3);
        assert_eq!(seen[0], vec![0, 1, 2]);
        // seen[1] must reflect the mutation window 0 made to the
        // overlapping elements (indices 1, 2) — proof this is a genuine
        // *lending* iterator over shared storage, not a copy.
        assert_eq!(seen[1], vec![101, 102, 3]);
        // seen[2] reflects window 1's mutation of the overlap (index 2, 3).
        assert_eq!(seen[2], vec![202, 103, 4]);
        // Final buffer carries every window's mutation.
        assert_eq!(data, [100, 201, 302, 203, 104]);
    }

    #[test]
    fn test_lending_windows_mut_too_short_yields_nothing() {
        let mut data = [1, 2];
        let mut windows = WindowsMut::new(&mut data, 3);
        assert!(windows.next().is_none());
    }
}
