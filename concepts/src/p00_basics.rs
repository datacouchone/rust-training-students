pub fn byte_count(text: &str) -> usize {
    // PUZZLE BEGIN demo
    todo!("p00_basics: demo")
    // PUZZLE END demo
}
pub fn greeting(name: &str) -> String {
    // PUZZLE BEGIN exercise
    todo!("p00_basics: exercise")
    // PUZZLE END exercise
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_borrow_and_unicode() {
        let text = String::from("Rust");
        assert_eq!(byte_count(&text), 4);
        assert_eq!(text, "Rust");
        assert_eq!(byte_count(""), 0);
        assert_eq!(byte_count("é"), 2);
    }
    #[test]
    fn exercise_owned_result_borrowed_input() {
        let name = String::from("Ada");
        assert_eq!(greeting(&name), "Hello, Ada!");
        assert_eq!(name, "Ada");
        assert_eq!(greeting(""), "Hello, !");
    }
}
