use std::cell::UnsafeCell;

struct Shared(UnsafeCell<u32>);
// DELIBERATE BUG: this promise has no synchronization to justify it.
unsafe impl Sync for Shared {}

impl Shared {
    fn overwrite(&self, value: u32) {
        // BUG: UnsafeCell permits interior mutation, not concurrent races.
        unsafe {
            *self.0.get() = value;
        }
    }
}

fn main() {
    assert!(
        cfg!(miri),
        "Intentional invalid example: run with cargo miri run --example 04_data_race"
    );
    let shared = Shared(UnsafeCell::new(0));
    std::thread::scope(|scope| {
        scope.spawn(|| shared.overwrite(1));
        scope.spawn(|| shared.overwrite(2));
    });
    // Scope joins both children; it does not order their writes to each other.
}
