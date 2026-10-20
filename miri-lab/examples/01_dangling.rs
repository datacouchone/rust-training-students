// Intentionally invalid. Inspect and execute only with Miri.
fn main() {
    assert!(
        cfg!(miri),
        "Intentional invalid example: run with cargo miri run --example 01_dangling"
    );
    let owner = Box::new(42u32);
    let address: *const u32 = &*owner;
    drop(owner);
    // BUG: the address remains, but its allocation has been freed.
    let value = unsafe { address.read() };
    println!("{value}");
}
