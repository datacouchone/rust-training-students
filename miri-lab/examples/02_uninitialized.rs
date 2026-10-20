use std::mem::MaybeUninit;

fn main() {
    assert!(
        cfg!(miri),
        "Intentional invalid example: run with cargo miri run --example 02_uninitialized"
    );
    let slot = MaybeUninit::<u32>::uninit();
    // BUG: storage exists, but no u32 has been written into it.
    let value = unsafe { slot.assume_init() };
    println!("{value}");
}
