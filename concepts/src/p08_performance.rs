pub fn chunk_sum(values: &[u32]) -> u64 {
    // PUZZLE BEGIN demo
    todo!("p08_performance: demo")
    // PUZZLE END demo
}
pub fn count_at_least(values: &[u32], threshold: u32) -> usize {
    // PUZZLE BEGIN exercise
    todo!("p08_performance: exercise")
    // PUZZLE END exercise
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_all_tail_lengths_and_wide_accumulation() {
        for len in 0..13 {
            let values: Vec<u32> = (0..len).map(|i| i * 7).collect();
            let expected: u64 = values.iter().map(|&v| u64::from(v)).sum();
            assert_eq!(chunk_sum(&values), expected);
        }
        assert_eq!(chunk_sum(&[u32::MAX; 5]), u64::from(u32::MAX) * 5);
    }
    #[test]
    fn exercise_threshold_boundaries() {
        assert_eq!(count_at_least(&[], 5), 0);
        assert_eq!(count_at_least(&[0, 4, 5, 6], 5), 2);
        assert_eq!(count_at_least(&[0, 4, 5, 6], 0), 4);
        assert_eq!(count_at_least(&[0, 4, 5, 6], u32::MAX), 0);
    }
}
