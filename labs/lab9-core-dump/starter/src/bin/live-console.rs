//! Loopback-only launcher for the live Tokio Console classroom demonstration.
//! Uses the actual lab server and fault implementations; no copied server logic.
use lab9_core_dump_starter::{AppState, Fault, init_tracing, run};
use tokio::net::TcpListener;

#[tokio::main(worker_threads = 2)]
async fn main() -> std::io::Result<()> {
    init_tracing();
    let port = std::env::var("METRICS_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(19090);
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    run(listener, AppState::new(Fault::from_env())).await
}
