//! Thin binary entry point. All logic lives in `lib.rs` and is exercised
//! directly by the crate's unit tests -- this file only wires the process
//! environment (env vars, the listening socket) to that logic.

use lab9_core_dump_starter::{AppState, Fault, init_tracing, run};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let fault = Fault::from_env();
    let state = AppState::new(fault);

    let port: u16 = std::env::var("METRICS_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9000);

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    run(listener, state).await
}
