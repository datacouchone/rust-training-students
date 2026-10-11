pub fn checked_ref<T>(values: &[T], index: usize) -> Option<&T> {
    // PUZZLE BEGIN demo
    todo!("p02_unsafe: demo")
    // PUZZLE END demo
}
pub fn checked_swap<T>(values: &mut [T], a: usize, b: usize) -> bool {
    // PUZZLE BEGIN exercise
    todo!("p02_unsafe: exercise")
    // PUZZLE END exercise
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_empty_and_bounds() {
        assert_eq!(checked_ref(&[10, 20], 1), Some(&20));
        assert_eq!(checked_ref(&[10, 20], 2), None);
        assert_eq!(checked_ref::<i32>(&[], 0), None);
        assert!(checked_ref(&[(), ()], 1).is_some());
    }
    #[test]
    fn exercise_swap_and_invalid_indices() {
        let mut v = [String::from("a"), String::from("b")];
        assert!(checked_swap(&mut v, 0, 1));
        assert_eq!(v, ["b", "a"]);
        assert!(checked_swap(&mut v, 0, 0));
        assert!(!checked_swap(&mut v, 0, 2));
        assert_eq!(v, ["b", "a"]);
        assert!(!checked_swap::<i32>(&mut [], 0, 0));
        assert!(checked_swap(&mut [(), ()], 0, 1));
    }
}
