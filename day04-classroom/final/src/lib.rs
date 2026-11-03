#![feature(portable_simd)]
//! A deliberately faulty gateway; real loopback TCP, bounded fixture, no Docker.
pub mod gateway;
pub mod policy;
pub mod telemetry;
pub mod transport;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub tenant: u32,
    pub path: String,
    pub payload: String,
    pub delay_ms: u64,
    pub fail_first: bool,
}
impl Request {
    pub fn sample(id: u64) -> Self {
        Self {
            id,
            tenant: 7,
            path: "/api/orders".into(),
            payload: "event".into(),
            delay_ms: 10,
            fail_first: false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    pub status: String,
    pub pool: Option<u16>,
    pub queue_us: u64,
    pub route_us: u64,
    pub upstream_us: u64,
    pub total_us: u64,
}
