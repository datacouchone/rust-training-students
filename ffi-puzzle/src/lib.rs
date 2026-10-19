//! Advanced FFI puzzle: borrowed foreign text and panic-safe updates.
//! Implement get_str and update_with; the completed basic wrapper is provided.
//! Native tests call C; Miri selects the Rust mock with the same API contract.

use std::ffi::{CStr, CString};
use std::fmt;
use std::ptr::NonNull;

// ---------------------------------------------------------------------
// FFI layer: real bindings, or a pure-Rust Miri mock with the same
// function signatures. Everything below this module only ever calls
// `ffi::legacy_*` and never knows (or needs to know) which backend it's
// talking to.
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
// LegacyDb
// ---------------------------------------------------------------------

/// A safe handle to a legacy database.
///
/// Owns exactly one `legacy_db*` for its whole lifetime. There is no
/// public way to duplicate or forge the pointer: [`LegacyDb`] is
/// not [`Clone`] or [`Copy`], and [`Drop`] closes it exactly once. See
/// [`test_double_close_impossible_by_construction`] for why this makes
/// double-close a compile error rather than a runtime bug.
pub struct LegacyDb(NonNull<legacy_db>);

impl LegacyDb {
    /// Borrow UTF-8 text directly from the C database without copying it.
    /// Invalid UTF-8 keys take precedence over interior-NUL errors.
    /// Missing keys return Ok(None); an empty stored value is Ok(Some("" )).
    /// The output must borrow the database, not the input key.
    ///
    /// A borrowed result prevents replacement while it is still used:
    /// ```compile_fail
    /// use ffi_puzzle::LegacyDb;
    /// let mut db = LegacyDb::open("borrow.db").unwrap();
    /// db.put(b"k", "old").unwrap();
    /// let value = db.get_str(b"k").unwrap().unwrap();
    /// db.put(b"k", "new").unwrap();
    /// println!("{value}");
    /// ```
    /// The owner cannot close while its borrowed text is still used:
    /// ```compile_fail
    /// use ffi_puzzle::LegacyDb;
    /// let mut db = LegacyDb::open("close.db").unwrap();
    /// db.put(b"k", "old").unwrap();
    /// let value = db.get_str(b"k").unwrap().unwrap();
    /// drop(db);
    /// println!("{value}");
    /// ```
    /// A temporary key does not constrain the returned value's lifetime:
    /// ```
    /// use ffi_puzzle::LegacyDb;
    /// let mut db = LegacyDb::open("temporary.db").unwrap();
    /// db.put(b"k", "value").unwrap();
    /// let value = {
    ///     let temporary_key = Vec::from(&b"k"[..]);
    ///     db.get_str(&temporary_key).unwrap().unwrap()
    /// };
    /// assert_eq!(value, "value");
    /// ```
    pub fn get_str<'db>(&'db self, key: &[u8]) -> Result<Option<&'db str>, DbError> {
        // PUZZLE: validate in the specified order, handle absence, and borrow
        // the foreign bytes as strict UTF-8. Never take ownership of C memory.
        // Do not call get(), allocate/copy a value, leak memory, or use 'static.
        // SAFETY: name the owner, terminator and invalidation rules.
        todo!("implement zero-copy get_str")
    }

    /// Read the old value, call transform once, then write the owned result.
    /// Invalid keys must be rejected before transform runs. A panic propagates
    /// without changing the database. Interior NUL in the result is an error
    /// and must leave the old value unchanged. Empty keys and values are valid.
    /// All borrows of foreign output must end before replacement occurs.
    pub fn update_with<F>(&mut self, key: &[u8], transform: F) -> Result<(), DbError>
    where
        F: FnOnce(Option<&str>) -> String,
    {
        // PUZZLE: compose get_str with the existing safe put API.
        // Do not copy the old value before calling transform or use new unsafe.
        // Do not catch/swallow callback panics or write a placeholder first.
        todo!("implement update_with")
    }

    /// Open (or create) the legacy database at `path`.
    pub fn open(path: &str) -> Result<Self, DbError> {
        let c_path = CString::new(path).map_err(|_| DbError::InteriorNul)?;

        // SAFETY: `c_path` is a valid, NUL-terminated C string kept alive
        // on our stack until after `ffi::legacy_open` returns.
        // `legacy_open` reads it synchronously and does not retain the
        // pointer past the call (see csrc/legacy.h).
        let raw = unsafe { ffi::legacy_open(c_path.as_ptr()) };

        NonNull::new(raw).map(LegacyDb).ok_or(DbError::OpenFailed)
    }

    /// Insert or overwrite the value stored under `key`.
    ///
    /// `key` is taken as raw bytes because this legacy format's keys
    /// originate outside Rust's type system (e.g. read off disk); it is
    /// validated as UTF-8, then as NUL-free, before ever reaching the C
    /// boundary.
    pub fn put(&mut self, key: &[u8], value: &str) -> Result<(), DbError> {
        if std::str::from_utf8(key).is_err() {
            return Err(DbError::InvalidUtf8);
        }
        let c_key = CString::new(key).map_err(|_| DbError::InteriorNul)?;
        let c_value = CString::new(value).map_err(|_| DbError::InteriorNul)?;

        // SAFETY: `self.0` is a live handle for as long as `self` exists
        // -- private ownership and exclusive access keep it live during the call.
        // `c_key`/`c_value` are valid, NUL-terminated C strings kept
        // alive on our stack until after the call. `legacy_put` copies
        // both strings internally and does not retain either pointer
        // past the call (see csrc/legacy.h).
        let rc = unsafe { ffi::legacy_put(self.0.as_ptr(), c_key.as_ptr(), c_value.as_ptr()) };

        if rc == 0 {
            Ok(())
        } else {
            Err(DbError::PutFailed)
        }
    }

    /// Look up the value stored under `key`, if any.
    pub fn get(&self, key: &[u8]) -> Option<String> {
        let key_str = std::str::from_utf8(key).ok()?;
        let c_key = CString::new(key_str).ok()?;

        // SAFETY: `self.0` is a live handle for as long as `self` exists.
        // `c_key` is a valid, NUL-terminated C string kept alive on our
        // stack until after the call.
        let raw = unsafe { ffi::legacy_get(self.0.as_ptr(), c_key.as_ptr()) };

        if raw.is_null() {
            return None;
        }

        // SAFETY: a non-null return from `legacy_get` points to a
        // NUL-terminated string owned by the database, valid until the
        // next `put` on this key or until `legacy_close` (see
        // csrc/legacy.h). We copy it into an owned `String` immediately
        // and never touch `raw` again, so nothing outlives the call.
        let value = unsafe { CStr::from_ptr(raw) };
        Some(value.to_string_lossy().into_owned())
    }
}

impl Drop for LegacyDb {
    fn drop(&mut self) {
        // SAFETY: `self.0` was produced by a successful `legacy_open` and
        // is closed exactly once, right here, as `self` is destroyed.
        // `LegacyDb` is not `Clone`/`Copy`, so no other copy of the
        // pointer can exist to double-close or use-after-free it.
        unsafe { ffi::legacy_close(self.0.as_ptr()) };
    }
}

/// Demonstrates that closing a [`LegacyDb`] twice is impossible by
/// construction: `LegacyDb` has no [`Clone`]/[`Copy`] impl, and
/// `std::mem::drop` consumes the wrapper value, so
/// the borrow checker refuses to compile any further use of a value that
/// has already been dropped.
///
/// ```compile_fail
/// use ffi_puzzle::LegacyDb;
///
/// let db = LegacyDb::open("doctest.db").unwrap();
/// drop(db);
/// drop(db); // error[E0382]: use of moved value: `db`
/// ```
#[allow(dead_code)]
fn test_double_close_impossible_by_construction() {}

// ---------------------------------------------------------------------
// Exposing Rust to C
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
    if ptr.is_null() || len == 0 {
        return 0;
    }

    // SAFETY: per the contract documented above, the caller guarantees
    // `ptr` is valid for `len` readable, initialized bytes for the
    // duration of this call.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    bytes.iter().fold(0u32, |acc, &b| {
        acc.wrapping_mul(31).wrapping_add(u32::from(b))
    })
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreign_invalid_utf8_is_an_error_not_lossy_text() {
        let mut db = LegacyDb::open("foreign-utf8.db").unwrap();
        let key = CString::new("raw").unwrap();
        let value = CString::new(vec![0xff]).unwrap();
        // SAFETY: live handle and valid terminated C strings. C strings may
        // contain non-UTF-8 bytes; put copies both inputs before returning.
        let rc = unsafe { ffi::legacy_put(db.0.as_ptr(), key.as_ptr(), value.as_ptr()) };
        assert_eq!(rc, 0);
        assert_eq!(db.get_str(b"raw"), Err(DbError::InvalidUtf8));
        let calls = std::cell::Cell::new(0);
        assert_eq!(db.update_with(b"raw", |_| {
            calls.set(calls.get() + 1);
            String::from("must not replace invalid output")
        }), Err(DbError::InvalidUtf8));
        assert_eq!(calls.get(), 0);

    }


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
