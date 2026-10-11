fn main() {
    println!("cargo:rerun-if-changed=csrc/legacy.c");
    println!("cargo:rerun-if-changed=csrc/legacy.h");

    // Miri interprets MIR; it cannot execute compiled, native C object
    // code. Under `cargo miri test` the crate compiles out the real
    // `unsafe extern "C"` bindings (see the `#[cfg(not(miri))] mod ffi`
    // block in src/lib.rs) and calls a pure-Rust mock with the same
    // function signatures instead (`#[cfg(miri)] mod ffi`). So there is
    // nothing for this build script to compile or link under Miri.
    if std::env::var_os("CARGO_CFG_MIRI").is_some() {
        return;
    }

    cc::Build::new()
        .file("csrc/legacy.c")
        .include("csrc")
        .warnings(true)
        .compile("legacy");
}
