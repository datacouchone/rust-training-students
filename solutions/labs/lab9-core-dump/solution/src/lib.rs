//! lab9-core-dump: a small async TCP metrics server used to teach outage
//! debugging with `tokio-console`, structured `tracing`, and core dumps.
//!
//! ## Wire protocol
//!
//! Length-prefixed text frames: a 4-byte big-endian length prefix followed
//! by that many bytes of UTF-8 text. A request frame's body is
//! `METRIC <name> <value>`; the server answers with one response frame per
//! request frame, body `OK` or `ERR <reason>`.
//!
//! ## Faults
//!
//! Behavior is healthy by default. Set `FAULT=deadlock`, `FAULT=task_leak`,
//! or `FAULT=panic` to inject one of the three faults this lab is built
//! around -- see [`fault::Fault`] and `server` for where each one lives.

pub mod fault;
pub mod frame;
pub mod server;
pub mod state;
pub mod tracing_init;

pub use fault::Fault;
pub use frame::{FrameError, Metric, parse_metric_frame, read_frame, write_frame};
pub use server::{handle_connection, run};
pub use state::AppState;
pub use tracing_init::init_tracing;
