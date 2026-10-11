use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
pub fn next_id(counter: &AtomicU64) -> u64 {
    // PUZZLE BEGIN demo
    todo!("p06_atomics: demo")
    // PUZZLE END demo
}
// Caller writes its payload before publishing; a reader accesses it only after
// observing ready. This flag alone does not make an arbitrary payload API safe.
pub fn mark_ready(flag: &AtomicBool) {
    // PUZZLE BEGIN exercise
    todo!("p06_atomics: exercise")
    // PUZZLE END exercise
}
pub fn is_ready(flag: &AtomicBool) -> bool {
    // PUZZLE BEGIN exercise
    todo!("p06_atomics: exercise")
    // PUZZLE END exercise
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_concurrent_ids() {
        let counter = AtomicU64::new(0);
        let mut ids = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|_| scope.spawn(|| (0..100).map(|_| next_id(&counter)).collect::<Vec<_>>()))
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        ids.sort_unstable();
        assert_eq!(ids, (0..400).collect::<Vec<_>>());
    }
    #[test]
    fn exercise_publication_flag() {
        let ready = AtomicBool::new(false);
        assert!(!is_ready(&ready));
        std::thread::scope(|scope| {
            scope.spawn(|| mark_ready(&ready)).join().unwrap();
        });
        assert!(is_ready(&ready));
    }
}
