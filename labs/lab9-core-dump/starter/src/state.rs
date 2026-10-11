//! Shared server state.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;

use crate::fault::Fault;

/// State shared across every connection.
///
/// `metrics` and `registry` are deliberately two independent
/// `tokio::sync::Mutex`es rather than one: the deadlock fault needs two
/// distinct locks to demonstrate opposite-order acquisition, and giving
/// the connection registry its own lock (rather than folding it into the
/// metrics map) mirrors how a real service tends to grow -- one lock per
/// subsystem, added independently over time, until two tasks each want
/// both in a different order.
#[derive(Clone)]
pub struct AppState {
    /// The fault selected for this process, if any. Read once at startup
    /// (see [`Fault::from_env`]) and threaded through as plain data.
    pub fault: Option<Fault>,
    /// name -> last reported value.
    pub metrics: Arc<Mutex<HashMap<String, f64>>>,
    /// IDs of currently-open connections.
    pub registry: Arc<Mutex<Vec<u64>>>,
    next_conn_id: Arc<AtomicU64>,
}

impl AppState {
    pub fn new(fault: Option<Fault>) -> Self {
        Self {
            fault,
            metrics: Arc::new(Mutex::new(HashMap::new())),
            registry: Arc::new(Mutex::new(Vec::new())),
            next_conn_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Allocate the next connection id (monotonically increasing, unique
    /// per process).
    pub fn next_conn_id(&self) -> u64 {
        self.next_conn_id.fetch_add(1, Ordering::Relaxed)
    }
}
