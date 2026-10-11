// These local Rust functions SIMULATE a C ABI. Real C linkage is in lab3.
// Contract: ptr is non-null, aligned, and valid for len initialized bytes
// for the duration of the call; len fits the slice size restrictions.
unsafe extern "C" fn foreign_sum(ptr: *const u8, len: usize) -> u64 {
    // SAFETY: the caller must satisfy the contract above.
    unsafe { std::slice::from_raw_parts(ptr, len) }
        .iter()
        .map(|&b| u64::from(b))
        .sum()
}
unsafe extern "C" fn foreign_max(ptr: *const u8, len: usize) -> u8 {
    // SAFETY: the caller must satisfy the contract above.
    unsafe { std::slice::from_raw_parts(ptr, len) }
        .iter()
        .copied()
        .max()
        .unwrap_or(0)
}
pub fn byte_total(bytes: &[u8]) -> u64 {
    // PUZZLE BEGIN demo
    todo!("p03_ffi: demo")
    // PUZZLE END demo
}
pub fn largest_byte(bytes: &[u8]) -> u8 {
    // PUZZLE BEGIN exercise
    todo!("p03_ffi: exercise")
    // PUZZLE END exercise
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_pointer_length_contract() {
        assert_eq!(byte_total(&[]), 0);
        assert_eq!(byte_total(&[255, 255, 1]), 511);
    }
    #[test]
    fn exercise_empty_and_nonempty() {
        assert_eq!(largest_byte(&[]), 0);
        assert_eq!(largest_byte(&[1, 255, 2]), 255);
        assert_eq!(largest_byte(&[0]), 0);
    }
}
