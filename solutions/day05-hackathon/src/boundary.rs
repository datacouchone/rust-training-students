//! Fixed-width wire fields are decoded explicitly; Rust/C padding is never serialized.
use std::ffi::{c_int, c_void};
unsafe extern "C" {
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}
/// Safe borrowed wrapper. No pointer escapes and C retains no ownership.
pub fn bytes_equal(a: &[u8], b: &[u8]) -> bool {
    // REPAIR: memory-safe prefix equality is not complete record equality.
    if a.len() != b.len() {
        return false;
    }
    let n = a.len();
    if n == 0 {
        return true;
    }
    // SAFETY: both slices expose at least n initialized bytes for this call.
    unsafe { memcmp(a.as_ptr().cast(), b.as_ptr().cast(), n) == 0 }
}
#[repr(C)]
pub struct NativeSummary {
    pub tenant: u32,
    pub count: u64,
}
// repr(C) defines an ABI layout; it does not define a portable wire encoding.
pub struct Untrusted(pub Vec<u8>);
#[derive(Debug, Clone)]
pub struct Validated {
    tenant: u32,
    id: u64,
    bytes: u64,
    body: Vec<u8>,
}
impl Validated {
    pub fn tenant(&self) -> u32 {
        self.tenant
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Invalid {
    Shape,
    Version,
    Path,
}
impl TryFrom<Untrusted> for Validated {
    type Error = Invalid;
    fn try_from(raw: Untrusted) -> Result<Self, Invalid> {
        let b = raw.0;
        if b.len() < 21 || b.len() > 149 {
            return Err(Invalid::Shape);
        }
        // REPAIR: only version 1 and an absolute UTF-8 path are accepted.
        if b[0] != 1 {
            return Err(Invalid::Version);
        }
        let path = std::str::from_utf8(&b[21..]).map_err(|_| Invalid::Path)?;
        if !path.starts_with('/') {
            return Err(Invalid::Path);
        }
        let tenant = u32::from_be_bytes(b[1..5].try_into().unwrap());
        let id = u64::from_be_bytes(b[5..13].try_into().unwrap());
        let bytes = u64::from_be_bytes(b[13..21].try_into().unwrap());
        Ok(Self {
            tenant,
            id,
            bytes,
            body: b,
        })
    }
}
/// Supplied fixture encoder: version + tenant + request ID + charge + path.
pub fn encode(tenant: u32, id: u64, bytes: u64, path: &str) -> Vec<u8> {
    let mut b = vec![1];
    b.extend(tenant.to_be_bytes());
    b.extend(id.to_be_bytes());
    b.extend(bytes.to_be_bytes());
    b.extend(path.as_bytes());
    b
}
