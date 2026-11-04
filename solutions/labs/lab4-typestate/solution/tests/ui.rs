//! trybuild harness: compiles the sources under `tests/ui/` and checks
//! their output against the matching `.stderr` file. This is what proves
//! the typestate guarantee is enforced *by the compiler*, not just by
//! convention: `send_on_closed.rs` must fail to compile.

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/send_on_closed.rs");
}
