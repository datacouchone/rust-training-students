fn main() {
    assert!(
        cfg!(miri),
        "Intentional invalid example: run with cargo miri run --example 03_out_of_bounds"
    );
    let values = [10u32, 20, 30];
    let pointer = values.as_ptr();
    // Forming a one-past pointer is allowed; reading there is not.
    let end = unsafe { pointer.add(values.len()) };
    let value = unsafe { end.read() };
    println!("{value}");
}
