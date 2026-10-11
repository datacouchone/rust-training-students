//! Integration tests for the `lab5-derive-macro-starter` crate.
//!
//! `FieldSummary` — the *trait* — is defined right here rather than in
//! `src/lib.rs`. A `proc-macro = true` crate can only export macros, so
//! whatever trait the generated `impl` block targets has to be supplied by
//! whoever invokes the derive. Real projects split this into a `-macros`
//! crate (generates the impl) plus a small runtime crate that owns the
//! trait, re-exported together from a facade crate. We keep it to one
//! crate here to stay focused on `syn`/`quote`; see the lab `README.md`
//! for the full rationale.
//!
//! `FieldSummary` the derive macro (imported below) and `FieldSummary` the
//! trait (declared below) don't collide: macros and types live in separate
//! namespaces in Rust, so both names can coexist in the same scope.
//!
//! The two `#[ignore]`d tests below will start passing once you fill in
//! the `// === YOUR CODE: emit the impl block ===` block in `src/lib.rs`
//! (until then, `summary()` calls `unimplemented!()` and panics at
//! runtime — the crate and these tests still compile fine). Run
//! `cargo test -p lab5-derive-macro-starter -- --ignored` to check your
//! work as you go.

use lab5_derive_macro_starter::{FieldSummary, timed};

trait FieldSummary {
    fn summary() -> String;
}

/// A minimal, dependency-free `block_on` for driving a `Future` to
/// completion in a test — no `tokio`/`futures` executor needed. Builds a
/// `Waker` (via `std::task::Wake`) that unparks this thread, then loops
/// `poll`/`park` until the future resolves. This is the standard few-line
/// std-only executor shape; real executors add work queues, I/O
/// readiness, etc., but the poll-park-wake loop underneath is the same.
fn block_on<F: std::future::Future>(mut fut: F) -> F::Output {
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    use std::thread::{self, Thread};

    struct ThreadWaker(Thread);

    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    // Safety: `fut` is a local that we never move again after this point;
    // pinning it in place on the stack is sound.
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => thread::park(),
        }
    }
}

// `summary()` is an associated function (no `&self`) that only inspects
// field *declarations*, not values — so these structs are never
// instantiated. `#[allow(dead_code)]` silences the resulting (expected)
// "field never read" warning.
#[derive(FieldSummary)]
#[allow(dead_code)]
struct Config {
    host: String,
    port: i32,
    #[summary(skip)]
    api_key: String,
}

#[test]
#[ignore = "TODO(lab5): fill in the YOUR CODE block in src/lib.rs, then rerun with --ignored"]
fn test_summary_lists_fields() {
    #[derive(FieldSummary)]
    #[allow(dead_code)]
    struct Point {
        x: i32,
        y: i32,
    }

    assert_eq!(Point::summary(), "x: i32, y: i32");
}

#[test]
#[ignore = "TODO(lab5): fill in the YOUR CODE block in src/lib.rs, then rerun with --ignored"]
fn test_skip_attribute_honored() {
    let summary = Config::summary();

    // Non-skipped fields are present, in declaration order.
    assert_eq!(summary, "host: String, port: i32");

    // The skipped field never shows up, under any spelling.
    assert!(!summary.contains("api_key"));
    assert!(!summary.contains("secret-looking-value"));
}

#[test]
fn test_timed_preserves_return_value() {
    // `#[timed]` is already fully implemented — this test passes as-is
    // and is here as a second illustration of the technique.
    #[timed]
    fn compute(x: u32, y: u32) -> u32 {
        x * y + 1
    }

    assert_eq!(compute(6, 7), 43);
}

#[test]
fn test_timed_preserves_early_return() {
    #[timed]
    fn classify(n: i32) -> &'static str {
        if n < 0 {
            return "negative";
        }
        if n == 0 {
            return "zero";
        }
        "positive"
    }

    assert_eq!(classify(-5), "negative");
    assert_eq!(classify(0), "zero");
    assert_eq!(classify(5), "positive");
}

#[test]
fn test_timed_works_on_async_fn() {
    // `#[timed]` is provided, fully-implemented code (like the rest of
    // this test), not part of the `YOUR CODE` exercise — so this test is
    // NOT `#[ignore]`d: it must pass on the starter as shipped, both
    // before and after you fill in the derive block. `fetch` has a
    // genuine `.await` (awaiting `std::future::ready`), which would fail
    // to *compile* under a naive synchronous-closure `#[timed]`
    // expansion — this exercises the provided async-aware wrapper.
    #[timed]
    async fn fetch(x: u32) -> u32 {
        std::future::ready(x * 2).await
    }

    assert_eq!(block_on(fetch(21)), 42);
}
