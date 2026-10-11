//! Lab 3: build a safe Rust wrapper around a mock "legacy" C key/value
//! store.
//!
//! The real C API lives in `csrc/legacy.h` / `csrc/legacy.c` and is
//! already built for you by `build.rs` via the `cc` crate -- you never
//! need to touch either of those files. Your job is the safe wrapper:
//! [`LegacyDb`]'s `open` / `put` / `get` / [`Drop`] methods, and one
//! small Rust-to-C export, [`rust_checksum`]. Every place you need to
//! write code is marked `todo!()`.
//!
//! # Miri
//!
//! Miri interprets MIR and cannot execute the natively compiled C code in
//! `csrc/legacy.c`. So the *only* thing that talks to real C
//! (`#[cfg(not(miri))] mod ffi`, below) is swapped out, at compile time,
//! for a pure-Rust mock with identical function signatures and semantics
//! (`#[cfg(miri)] mod ffi`, also below). This infrastructure is given to
//! you complete -- the wrapper code you write calls `ffi::legacy_*`
//! either way and never needs a `#[cfg(miri)]` of its own.

use std::ffi::{CStr, CString};
use std::fmt;
use std::ptr::NonNull;

// ---------------------------------------------------------------------
// FFI layer: real bindings, or a pure-Rust Miri mock with the same
// function signatures. This part is given to you -- attendees never
// write C, and never need their own `#[cfg(miri)]`. Everything below
// this module only ever calls `ffi::legacy_*` and never knows (or needs
// to know) which backend it's talking to.
// ---------------------------------------------------------------------

#[cfg(not(miri))]
mod ffi {
    //! Real bindings to the C library built from `csrc/legacy.c`.
    use std::ffi::{c_char, c_int};

    /// Opaque handle to a `legacy_db`. Rust never looks inside this type
    /// -- it only ever holds a `NonNull<legacy_db>` and hands the pointer
    /// back to the C library. This is the Rustonomicon's recommended
    /// "opaque FFI type" idiom: an uninstantiable, zero-sized marker that
    /// also opts the type out of `Send` / `Sync` / `Unpin` via
    /// `PhantomData`, so we never accidentally claim more than the real C
    /// type guarantees.
    #[repr(C)]
    pub struct legacy_db {
        _data: [u8; 0],
        _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
    }

    unsafe extern "C" {
        pub fn legacy_open(path: *const c_char) -> *mut legacy_db;
        pub fn legacy_put(db: *mut legacy_db, k: *const c_char, v: *const c_char) -> c_int;
        pub fn legacy_get(db: *mut legacy_db, k: *const c_char) -> *const c_char;
        pub fn legacy_close(db: *mut legacy_db);
    }
}

#[cfg(miri)]
mod ffi {
    //! Pure-Rust stand-in for `csrc/legacy.c`, compiled only under Miri.
    //!
    //! Same function names, same signatures, same pointer-ownership
    //! contract as the real C library (documented in `csrc/legacy.h`):
    //! `legacy_open` hands out a handle that must be closed exactly once;
    //! `legacy_get` returns a pointer owned by the db, valid until the
    //! next `put` on that key or until `legacy_close`, that the caller
    //! must not free.
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::ffi::{CStr, CString, c_char, c_int};

    // The name intentionally matches the real C type in csrc/legacy.h
    // (see the `#[cfg(not(miri))]` module above) so both `ffi::legacy_db`
    // definitions are interchangeable from the wrapper's point of view;
    // it's not `#[repr(C)]` because, unlike the real one, this mock never
    // crosses the FFI boundary.
    #[allow(non_camel_case_types)]
    pub struct legacy_db {
        entries: RefCell<HashMap<String, CString>>,
    }

    /// # Safety
    /// Mirrors the real `legacy_open`: `path` must be null, or a valid
    /// pointer to a NUL-terminated C string.
    pub unsafe fn legacy_open(path: *const c_char) -> *mut legacy_db {
        if path.is_null() {
            return std::ptr::null_mut();
        }
        // SAFETY: caller guarantees `path` is a valid NUL-terminated C
        // string; we only validate it (mirroring what the real
        // `legacy_open` does when it copies `path`) and don't retain it.
        let _ = unsafe { CStr::from_ptr(path) };
        Box::into_raw(Box::new(legacy_db {
            entries: RefCell::new(HashMap::new()),
        }))
    }

    /// # Safety
    /// Mirrors the real `legacy_put`: `db` must be a live pointer
    /// returned by `legacy_open` and not yet closed; `k` and `v` must be
    /// valid NUL-terminated C strings.
    pub unsafe fn legacy_put(db: *mut legacy_db, k: *const c_char, v: *const c_char) -> c_int {
        if db.is_null() || k.is_null() || v.is_null() {
            return -1;
        }
        // SAFETY: caller guarantees `db` is live and `k`/`v` are valid
        // NUL-terminated C strings for the duration of this call.
        let db = unsafe { &*db };
        let key = unsafe { CStr::from_ptr(k) }.to_string_lossy().into_owned();
        let value = unsafe { CStr::from_ptr(v) }.to_owned();
        db.entries.borrow_mut().insert(key, value);
        0
    }

    /// # Safety
    /// Mirrors the real `legacy_get`: `db` must be a live pointer
    /// returned by `legacy_open`; `k` must be a valid NUL-terminated C
    /// string. The returned pointer, if non-null, is owned by `db` (do
    /// not free it) and stays valid until the next `legacy_put` for the
    /// same key or until `legacy_close`.
    pub unsafe fn legacy_get(db: *mut legacy_db, k: *const c_char) -> *const c_char {
        if db.is_null() || k.is_null() {
            return std::ptr::null();
        }
        // SAFETY: caller guarantees `db` is live and `k` is a valid
        // NUL-terminated C string for the duration of this call.
        let db = unsafe { &*db };
        let key = unsafe { CStr::from_ptr(k) }.to_string_lossy().into_owned();
        match db.entries.borrow().get(&key) {
            Some(value) => value.as_ptr(),
            None => std::ptr::null(),
        }
    }

    /// # Safety
    /// Mirrors the real `legacy_close`: `db` must be null, or a pointer
    /// previously returned by `legacy_open` that has not already been
    /// closed. `db` must never be used again after this call.
    pub unsafe fn legacy_close(db: *mut legacy_db) {
        if !db.is_null() {
            // SAFETY: caller guarantees `db` was produced by `legacy_open`
            // (i.e. by `Box::into_raw` above) and is being closed exactly
            // once here.
            drop(unsafe { Box::from_raw(db) });
        }
    }
}

use ffi::legacy_db;

// ---------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------

/// Errors returned by the safe [`LegacyDb`] wrapper.
#[derive(Debug, PartialEq, Eq)]
pub enum DbError {
    /// A string destined for the C library contained an interior NUL
    /// byte and cannot be represented as a C string.
    InteriorNul,
    /// A key was not valid UTF-8. This legacy store only ever indexes
    /// text keys, so the byte string is rejected before it can cross
    /// into C.
    InvalidUtf8,
    /// `legacy_open` returned NULL (allocation failure inside the C
    /// library).
    OpenFailed,
    /// `legacy_put` reported failure (returned non-zero).
    PutFailed,
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            DbError::InteriorNul => "string contained an interior NUL byte",
            DbError::InvalidUtf8 => "key was not valid UTF-8",
            DbError::OpenFailed => "legacy_open failed to allocate a handle",
            DbError::PutFailed => "legacy_put reported failure",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for DbError {}

// ---------------------------------------------------------------------
// LegacyDb -- YOUR CODE GOES HERE
// ---------------------------------------------------------------------

/// A safe handle to a legacy database.
///
/// Owns exactly one `legacy_db*` for its whole lifetime. There is no
/// public way to duplicate, forge, or leak the pointer: [`LegacyDb`] is
/// not [`Clone`] or [`Copy`], and [`Drop`] closes it exactly once. See
/// [`test_double_close_impossible_by_construction`] for why this makes
/// double-close a compile error rather than a runtime bug.
pub struct LegacyDb(NonNull<legacy_db>);

impl LegacyDb {
    /// Open (or create) the legacy database at `path`.
    pub fn open(path: &str) -> Result<Self, DbError> {
        // TODO:
        //  1. Convert `path` to a `CString` (reject an interior NUL byte
        //     as `DbError::InteriorNul` via `?`).
        //  2. Call `ffi::legacy_open` with the C string's pointer.
        //  3. Wrap the returned pointer in a `NonNull` (treat a NULL
        //     return as `DbError::OpenFailed`) and construct `Self`.
        //
        // SAFETY: TODO -- once you write the `unsafe { ffi::legacy_open(..) }`
        // call, justify it here: how long does the C string pointer you
        // pass in need to stay valid, and for how long does it actually
        // stay valid? What does `legacy_open` promise about the pointer
        // it hands back (see csrc/legacy.h)?
        todo!("open: convert path, call ffi::legacy_open, wrap the result in NonNull")
    }

    /// Insert or overwrite the value stored under `key`.
    ///
    /// `key` is taken as raw bytes because this legacy format's keys
    /// originate outside Rust's type system (e.g. read off disk); it
    /// should be validated as UTF-8, then as NUL-free, before ever
    /// reaching the C boundary.
    pub fn put(&mut self, key: &[u8], value: &str) -> Result<(), DbError> {
        // TODO:
        //  1. Reject `key` that isn't valid UTF-8 with `DbError::InvalidUtf8`
        //     (`std::str::from_utf8`).
        //  2. Convert `key` and `value` to `CString`s (reject an interior
        //     NUL byte in either as `DbError::InteriorNul`).
        //  3. Call `ffi::legacy_put`; translate a non-zero return into
        //     `DbError::PutFailed`.
        //
        // SAFETY: TODO -- what does `self.0` being a live, exclusively-
        // owned pointer buy you here? How long do the two `CString`s need
        // to stay alive relative to the call?
        todo!("put: validate UTF-8, build two CStrings, call ffi::legacy_put")
    }

    /// Look up the value stored under `key`, if any.
    pub fn get(&self, key: &[u8]) -> Option<String> {
        // TODO:
        //  1. Validate `key` as UTF-8 and build a `CString` from it,
        //     returning `None` early (via `?` on `Option`) on either
        //     failure -- a key that was never valid text was never
        //     stored.
        //  2. Call `ffi::legacy_get`; a NULL return means "not found".
        //  3. Otherwise wrap the returned pointer in a `CStr` and copy it
        //     into an owned `String` before returning it.
        //
        // SAFETY: TODO -- per csrc/legacy.h, how long does the pointer
        // `legacy_get` returns stay valid, and who owns it? Why is it
        // important that you copy it into a `String` before this
        // function returns rather than handing the raw pointer back?
        todo!("get: validate UTF-8, build a CString, call ffi::legacy_get, copy the result")
    }
}

impl Drop for LegacyDb {
    fn drop(&mut self) {
        // TODO: call `ffi::legacy_close` on the wrapped pointer.
        //
        // SAFETY: TODO -- why is this guaranteed to run at most once per
        // handle? (Hint: what traits does `LegacyDb` *not* implement?)
        todo!("drop: call ffi::legacy_close exactly once")
    }
}

/// Demonstrates that closing a [`LegacyDb`] twice is impossible by
/// construction: `LegacyDb` has no [`Clone`]/[`Copy`] impl, and
/// [`Drop::drop`] takes `&mut self`, consuming access to the value, so
/// the borrow checker refuses to compile any further use of a value that
/// has already been dropped.
///
/// This doctest is given to you complete -- it doesn't depend on your
/// `open`/`put`/`get` implementations, only on `LegacyDb` not being
/// `Clone`/`Copy`, so it should already pass.
///
/// ```compile_fail
/// use lab3_ffi_starter::LegacyDb;
///
/// let db = LegacyDb::open("doctest.db").unwrap();
/// drop(db);
/// drop(db); // error[E0382]: use of moved value: `db`
/// ```
#[allow(dead_code)]
fn test_double_close_impossible_by_construction() {}

// ---------------------------------------------------------------------
// Exposing Rust to C -- YOUR CODE GOES HERE
// ---------------------------------------------------------------------

/// A small, deterministic checksum, exposed with a C-compatible ABI so it
/// can be called from C code the way `csrc/legacy.c` is called from Rust,
/// just in the opposite direction: `uint32_t rust_checksum(const uint8_t
/// *ptr, size_t len);`.
///
/// Marked `unsafe`, not just `extern "C"`: it dereferences a raw pointer
/// whose validity only the caller can guarantee, and Rust's own
/// `clippy::not_unsafe_ptr_arg_deref` (a deny-by-default *correctness*
/// lint) exists specifically to catch exported pointer-taking functions
/// that omit this. The `unsafe` qualifier is Rust-side only -- it changes
/// nothing about the exported symbol name, calling convention, or
/// parameter/return types a C caller sees, so it costs nothing at the ABI
/// boundary while forcing any *Rust* caller to open an `unsafe` block and
/// accept the contract below.
///
/// # Safety
///
/// If `ptr` is non-null, it must point to at least `len` readable,
/// initialized bytes, valid for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_checksum(ptr: *const u8, len: usize) -> u32 {
    // TODO:
    //  1. Treat a null pointer or zero length as checksum `0`.
    //  2. Otherwise, build a `&[u8]` over `ptr..ptr + len` and fold it
    //     into a `u32` however you like (a simple multiply-and-add is
    //     fine -- this isn't a cryptographic checksum).
    //
    // SAFETY: TODO -- this is the one function in this file where *you*
    // are the one making the unsafe promise to the compiler rather than
    // just relying on one the C library already made. What exactly are
    // you asserting about `ptr` and `len` when you build the slice?
    let _ = (ptr, len);
    todo!("rust_checksum: fold over the bytes at ptr..ptr+len")
}

// ---------------------------------------------------------------------
// Tests -- make these pass. Do not rename them.
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_put_get_roundtrip() {
        let mut db = LegacyDb::open("test-roundtrip.db").expect("open should succeed");
        db.put(b"language", "rust").expect("put should succeed");
        assert_eq!(db.get(b"language"), Some("rust".to_string()));
    }

    #[test]
    fn test_get_missing_returns_none() {
        let db = LegacyDb::open("test-missing.db").expect("open should succeed");
        assert_eq!(db.get(b"does-not-exist"), None);
    }

    #[test]
    fn test_invalid_utf8_key_rejected() {
        let mut db = LegacyDb::open("test-invalid-utf8.db").expect("open should succeed");
        // 0xFF is never valid in UTF-8, so this byte string can never
        // have been stored in a text-indexed legacy store.
        let bad_key: &[u8] = &[0xFF, 0xFE, 0xFD];

        let put_result = db.put(bad_key, "value");
        assert_eq!(put_result, Err(DbError::InvalidUtf8));

        // Never reached C at all -- correctly reported as absent rather
        // than panicking or smuggling invalid UTF-8 across the boundary.
        assert_eq!(db.get(bad_key), None);
    }

    #[test]
    fn test_interior_nul_byte_rejected() {
        let mut db = LegacyDb::open("test-interior-nul.db").expect("open should succeed");

        // "has\0nul" is valid UTF-8 but cannot be represented as a
        // NUL-terminated C string -- a different failure mode than
        // invalid UTF-8, and `CString::new` is what catches it.
        assert_eq!(db.put(b"has\0nul", "value"), Err(DbError::InteriorNul));
        assert_eq!(db.put(b"key", "has\0nul"), Err(DbError::InteriorNul));

        // `LegacyDb::open` goes through the same `CString::new` check.
        // (`LegacyDb` isn't `Debug`/`PartialEq` -- it's a raw-pointer
        // handle, not a value type -- so `matches!` rather than
        // `assert_eq!` here.)
        assert!(matches!(
            LegacyDb::open("has\0nul"),
            Err(DbError::InteriorNul)
        ));
    }

    #[test]
    fn test_rust_checksum_from_c_signature() {
        // This only type-checks if `rust_checksum` really has the
        // `unsafe extern "C" fn(*const u8, usize) -> u32` signature a C
        // caller would need -- proving ABI compatibility at compile time.
        let f: unsafe extern "C" fn(*const u8, usize) -> u32 = rust_checksum;

        let data = b"abcd";
        // SAFETY: `data` is a valid, initialized, `data.len()`-byte slice
        // for the duration of this call.
        let checksum = unsafe { f(data.as_ptr(), data.len()) };
        let expected = data.iter().fold(0u32, |acc, &b| {
            acc.wrapping_mul(31).wrapping_add(u32::from(b))
        });
        assert_eq!(checksum, expected);

        // Degenerate inputs must not read through a dangling/null pointer.
        // SAFETY: `len == 0` makes this a no-op regardless of `ptr`.
        assert_eq!(unsafe { f(std::ptr::null(), 0) }, 0);
    }
}
