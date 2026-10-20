use std::sync::{
    Barrier,
    atomic::{AtomicUsize, Ordering},
};

fn main() {
    let count = AtomicUsize::new(0);
    let both_have_read = Barrier::new(2);
    std::thread::scope(|scope| {
        for _ in 0..2 {
            scope.spawn(|| {
                let old = count.load(Ordering::SeqCst);
                // Both workers read 0 before either is allowed to store.
                both_have_read.wait();
                count.store(old + 1, Ordering::SeqCst);
            });
        }
    });
    let actual = count.load(Ordering::SeqCst);
    println!("Two increments: expected 2, observed {actual}");
    // Safe Rust, atomic accesses, incorrect algorithm: ordinary panic, not UB.
    assert_eq!(actual, 2);
}
