//! Independent transfer puzzles. Keep tests unchanged.
use std::ffi::{CString, NulError};

// E01: return (size, alignment, offsets in declaration order).
// Use repr(C) and std::mem measurement APIs for this exact type.
#[repr(C)]
pub struct Record {
    pub tag: u8,
    pub sequence: u32,
    pub code: u16,
}
pub fn record_layout() -> (usize, usize, [usize; 3]) {
    todo!("E01: measure Record")
}

// E02: explicit portable encoding: one tag byte, four big-endian
// sequence bytes, two big-endian code bytes. Do not copy struct memory.
pub fn encode_record(tag: u8, sequence: u32, code: u16) -> [u8; 7] {
    todo!("E02: encode each field")
}

// E04: `storage` models initialized backing storage, but only its prefix
// [0, live_len) is logically live. Refuse invalid live_len and spare slots.
// Implement this safely. An unsafe rewrite is optional after the contract works.
pub fn live_get<T>(storage: &[T], live_len: usize, index: usize) -> Option<&T> {
    todo!("E04: use the live range")
}

// E08: validate exactly the CString input contract, including empty input.
pub fn c_input(text: &str) -> Result<CString, NulError> {
    todo!("E08: preserve error information")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn e01_layout() {
        // These integer layout expectations apply to the course targets.
        assert_eq!(record_layout(), (12, 4, [0, 4, 8]));
    }
    #[test]
    fn e02_wire_bytes() {
        assert_eq!(encode_record(0xAA, 0x01020304, 0x0506),
                   [0xAA, 1, 2, 3, 4, 5, 6]);
        assert_eq!(encode_record(0, 0, 0), [0; 7]);
        assert_eq!(encode_record(255, u32::MAX, u16::MAX), [255; 7]);
    }
    #[test]
    fn e04_live_bounds() {
        assert_eq!(live_get(&[10, 20, 99, 99], 2, 1), Some(&20));
        assert_eq!(live_get(&[10, 20, 99, 99], 2, 2), None);
        assert_eq!(live_get(&[10], 2, 0), None);
        assert_eq!(live_get::<u32>(&[], 0, 0), None);
        assert_eq!(live_get(&[10], 0, 0), None);
        assert_eq!(live_get(&[10], 1, usize::MAX), None);
    }
    #[test]
    fn e08_c_strings() {
        assert_eq!(c_input("Rust").unwrap().as_bytes_with_nul(), b"Rust\0");
        assert_eq!(c_input("").unwrap().as_bytes_with_nul(), b"\0");
        assert!(c_input("has\0nul").is_err());
        assert_eq!(c_input("é").unwrap().as_bytes_with_nul(), &[0xc3, 0xa9, 0]);
    }
}
