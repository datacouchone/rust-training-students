//! `prereq-check` — a 30-minute, no-hints self-check for the prerequisite
//! this course assumes: "1-2 years of production Rust, with a thorough,
//! practical understanding of ownership, borrowing, and lifetimes."
//!
//! **Default build (no features):** this crate is an intentional no-op.
//! `cargo build -p prereq-check` (and `cargo build --workspace`) must stay
//! green so this self-check never blocks the other 9 labs.
//!
//! **Challenge build:** `cargo build -p prereq-check --features challenge`
//! pulls in four small, independently broken items below. Each fails to
//! compile for a different, ordinary "I've shipped Rust for a while" reason
//! — not obscure trivia. See `README.md` for the rules and the clock, and
//! Work through the challenge items and verify the resulting behavior.

#[cfg(not(feature = "challenge"))]
pub fn placeholder() {}

#[cfg(feature = "challenge")]
use std::collections::HashMap;

// ---------------------------------------------------------------------
// Exercise 1 — a function returning a reference tied to the wrong lifetime
// ---------------------------------------------------------------------
//
// `first_line` should return the first line of `text`, trimmed, borrowed
// from `text` itself. Fix the body (the signature is already correct).
#[cfg(feature = "challenge")]
pub fn first_line<'a>(text: &'a str) -> &'a str {
    let owned: String = text.lines().next().unwrap_or("").to_string();
    owned.trim()
}

// ---------------------------------------------------------------------
// Exercise 2 — an iterator adapter needing a lifetime annotation
// ---------------------------------------------------------------------
//
// `WithPrefix` wraps a `&str`-yielding iterator and yields only the
// tokens that start with `prefix`.
#[cfg(feature = "challenge")]
pub struct WithPrefix<I> {
    inner: I,
    prefix: &str,
}

#[cfg(feature = "challenge")]
impl<I> WithPrefix<I> {
    pub fn new(inner: I, prefix: &str) -> Self {
        WithPrefix { inner, prefix }
    }
}

#[cfg(feature = "challenge")]
impl<I> Iterator for WithPrefix<I>
where
    I: Iterator<Item = &str>,
{
    type Item = &str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.by_ref().find(|tok| tok.starts_with(self.prefix))
    }
}

// ---------------------------------------------------------------------
// Exercise 3 — a trait-object lifetime bound
// ---------------------------------------------------------------------
//
// `banned_word_validator` builds a `Validator` trait object that rejects
// any input containing one of the given banned words.
#[cfg(feature = "challenge")]
pub trait Validator {
    fn validate(&self, input: &str) -> bool;
}

#[cfg(feature = "challenge")]
struct BannedWords<'a> {
    banned: &'a [&'a str],
}

#[cfg(feature = "challenge")]
impl<'a> Validator for BannedWords<'a> {
    fn validate(&self, input: &str) -> bool {
        !self.banned.iter().any(|w| input.contains(w))
    }
}

#[cfg(feature = "challenge")]
pub fn banned_word_validator(banned: &[&str]) -> Box<dyn Validator> {
    Box::new(BannedWords { banned })
}

// ---------------------------------------------------------------------
// Exercise 4 — a borrow-across-mutation conflict needing restructuring
// ---------------------------------------------------------------------
//
// `get_or_insert` should return the existing value for `key`, or insert
// `default` and return that. The fix here is not "add another lifetime"
// — it's a different, smaller approach entirely.
#[cfg(feature = "challenge")]
pub fn get_or_insert<'a>(
    map: &'a mut HashMap<String, u64>,
    key: &str,
    default: u64,
) -> &'a u64 {
    if let Some(v) = map.get(key) {
        return v;
    }
    map.insert(key.to_string(), default);
    map.get(key).unwrap()
}

#[cfg(all(test, feature = "challenge"))]
mod tests {
    use super::*;

    #[test]
    fn test_first_line() {
        assert_eq!(first_line("  hello world  \nsecond line"), "hello world");
        assert_eq!(first_line(""), "");
    }

    #[test]
    fn test_with_prefix() {
        let tokens = vec!["rust_fn", "go_fn", "rust_macro", "python_fn"];
        let got: Vec<&str> = WithPrefix::new(tokens.into_iter(), "rust_").collect();
        assert_eq!(got, vec!["rust_fn", "rust_macro"]);
    }

    #[test]
    fn test_banned_word_validator() {
        let banned = ["heck", "darn"];
        let validator = banned_word_validator(&banned);
        assert!(validator.validate("a perfectly nice sentence"));
        assert!(!validator.validate("oh heck no"));
    }

    #[test]
    fn test_get_or_insert() {
        let mut map = HashMap::new();
        assert_eq!(*get_or_insert(&mut map, "a", 1), 1);
        assert_eq!(*get_or_insert(&mut map, "a", 99), 1);
        assert_eq!(*get_or_insert(&mut map, "b", 2), 2);
    }
}
