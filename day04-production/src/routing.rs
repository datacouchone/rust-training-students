//! Read-mostly, exact-host routing. HTTP/TLS parsing and proxying are upstream.
//! A request pins ONE configuration generation for all its policy decisions.
use arc_swap::ArcSwap;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Destination {
    pub pool: u16,
    pub timeout_ms: u32,
}
#[derive(Debug)]
pub struct Snapshot {
    pub generation: u64,
    routes: HashMap<String, Destination>,
}
impl Snapshot {
    /// Compile in the control plane, before taking a publication lock.
    /// Input hosts must already be ASCII lowercase without ports or wildcards.
    pub fn compile(
        generation: u64,
        rows: Vec<(String, Destination)>,
    ) -> Result<Self, &'static str> {
        if rows.is_empty() || rows.len() > 100_000 {
            return Err("route count");
        }
        let mut routes = HashMap::with_capacity(rows.len());
        for (host, destination) in rows {
            if host.is_empty()
                || host.len() > 253
                || !host
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'.' || c == b'-')
            {
                return Err("host must be normalized before compilation");
            }
            if destination.timeout_ms == 0 || destination.timeout_ms > 60_000 {
                return Err("timeout");
            }
            if routes.insert(host, destination).is_some() {
                return Err("duplicate host");
            }
        }
        Ok(Self { generation, routes })
    }
    pub fn route(&self, host: &str) -> Option<&Destination> {
        self.routes.get(host)
    }
}

pub struct Locked(pub RwLock<Arc<Snapshot>>);
impl Locked {
    pub fn pin(&self) -> Arc<Snapshot> {
        Arc::clone(&self.0.read().expect("configuration lock poisoned"))
    }
}
pub struct Published(pub ArcSwap<Snapshot>);
impl Published {
    pub fn new(snapshot: Snapshot) -> Self {
        Self(ArcSwap::from_pointee(snapshot))
    }
    /// Single control-plane writer; validation failure keeps the old generation.
    /// Generation is a label, not a multi-writer ordering/consensus protocol.
    pub fn reload(
        &self,
        generation: u64,
        rows: Vec<(String, Destination)>,
    ) -> Result<(), &'static str> {
        let next = Snapshot::compile(generation, rows)?;
        self.0.store(Arc::new(next));
        Ok(())
    }
}
pub fn fixture(generation: u64, n: usize) -> Vec<(String, Destination)> {
    (0..n)
        .map(|i| {
            (
                format!("tenant-{i}.example.net"),
                Destination {
                    pool: (i % 32) as u16,
                    timeout_ms: 100 + generation as u32 % 1000,
                },
            )
        })
        .collect()
}
