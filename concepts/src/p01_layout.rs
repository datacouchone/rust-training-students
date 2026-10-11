pub fn field_padding(offset: usize, alignment: usize) -> usize {
    assert!(alignment.is_power_of_two());
    // PUZZLE BEGIN demo
    todo!("p01_layout: demo")
    // PUZZLE END demo
}
pub fn padded_size(end: usize, alignment: usize) -> usize {
    assert!(alignment.is_power_of_two());
    // PUZZLE BEGIN exercise
    todo!("p01_layout: exercise")
    // PUZZLE END exercise
}
#[repr(C)]
pub struct Header {
    pub flag: u8,
    pub timestamp: u64,
    pub kind: u16,
    pub length: u32,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_boundary_offsets() {
        for (offset, alignment, expected) in
            [(0, 8, 0), (1, 8, 7), (8, 8, 0), (10, 4, 2), (3, 1, 0)]
        {
            assert_eq!(field_padding(offset, alignment), expected);
        }
    }
    #[test]
    fn exercise_tail_padding() {
        for (end, alignment, expected) in
            [(0, 8, 0), (15, 8, 16), (16, 8, 16), (17, 8, 24), (3, 1, 3)]
        {
            assert_eq!(padded_size(end, alignment), expected);
        }
    }
}
