//! Global tracing setup.

use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

/// Initialize global tracing.
///
/// Plain runs get structured `fmt` logs only, filtered by `RUST_LOG`
/// (defaulting to `info`). Setting `TOKIO_CONSOLE=1` additionally spawns
/// the `console-subscriber` layer, exposing the process to the
/// `tokio-console` CLI over its gRPC port.
///
/// `console-subscriber` requires the binary to be *built* with
/// `RUSTFLAGS="--cfg tokio_unstable"` (task-level instrumentation is a
/// Tokio unstable feature). Without that flag it panics on init with an
/// explanatory message telling you exactly that. Because this whole branch
/// is behind the `TOKIO_CONSOLE` gate, plain `cargo test` / `cargo run`
/// builds never hit it and need no special flags at all.
pub fn init_tracing() {
    // Built fresh per branch (rather than shared across both) so each
    // `Filtered<..., S>`'s subscriber type `S` is inferred independently --
    // sharing one instance would force both branches to agree on a single
    // `S`, which they can't (one includes the console layer, one doesn't).
    fn env_filter() -> EnvFilter {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    }

    let console_enabled = std::env::var("TOKIO_CONSOLE").as_deref() == Ok("1");

    if console_enabled {
        tracing_subscriber::registry()
            .with(console_subscriber::spawn())
            .with(tracing_subscriber::fmt::layer().with_filter(env_filter()))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_filter(env_filter()))
            .init();
    }
}
