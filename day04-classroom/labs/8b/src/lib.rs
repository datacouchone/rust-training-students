//! Bounded batch storage with deduplicated paths and explicit retention after flush.
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    ops::Range,
};
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub rows: usize,
    pub bytes: usize,
    pub paths: usize,
    pub retain_rows: usize,
    pub retain_bytes: usize,
    pub retain_buckets: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            rows: 4096,
            bytes: 256 * 1024,
            paths: 1024,
            retain_rows: 4096,
            retain_bytes: 256 * 1024,
            retain_buckets: 2048,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ticket {
    epoch: u64,
    row: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    RowLimit,
    ByteLimit,
    PathLimit,
    Overflow,
    InvalidPath,
    EpochExhausted,
}
#[derive(Clone, Debug)]
struct Row {
    tenant: u32,
    path: Range<usize>,
    bytes: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub struct View<'a> {
    pub tenant: u32,
    pub path: &'a str,
    pub bytes: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Footprint {
    pub payload_bytes: usize,
    pub unique_paths: usize,
    pub rows: usize,
    pub payload_capacity: usize,
    pub row_capacity: usize,
    pub bucket_capacity: usize,
}
pub struct Batch {
    limits: Limits,
    epoch: u64,
    storage: Vec<u8>,
    rows: Vec<Row>,
    paths: HashMap<u64, Vec<Range<usize>>>,
    unique: usize,
    total: u64,
    hash: fn(&str) -> u64,
}
pub fn path_hash(path: &str) -> u64 {
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    h.finish()
}
impl Batch {
    pub fn new(limits: Limits) -> Self {
        Self::with_hasher(limits, path_hash)
    }
    /// Hasher injection exists so tests can exercise collision correctness deterministically.
    pub fn with_hasher(limits: Limits, hash: fn(&str) -> u64) -> Self {
        Self {
            limits,
            epoch: 0,
            storage: vec![],
            rows: vec![],
            paths: HashMap::new(),
            unique: 0,
            total: 0,
            hash,
        }
    }
    pub fn total_bytes(&self) -> u64 {
        self.total
    }
    pub fn footprint(&self) -> Footprint {
        Footprint {
            payload_bytes: self.storage.len(),
            unique_paths: self.unique,
            rows: self.rows.len(),
            payload_capacity: self.storage.capacity(),
            row_capacity: self.rows.capacity(),
            bucket_capacity: self.paths.capacity(),
        }
    }
    pub fn push(&mut self, tenant: u32, path: &str, bytes: u64) -> Result<Ticket, Error> {
        // TASK_BEGIN push
        if !path.starts_with('/') {
            return Err(Error::InvalidPath);
        }
        if self.rows.len() >= self.limits.rows {
            return Err(Error::RowLimit);
        }
        let total = self.total.checked_add(bytes).ok_or(Error::Overflow)?;
        let hash = (self.hash)(path);
        let existing = self
            .paths
            .get(&hash)
            .and_then(|ranges| {
                ranges
                    .iter()
                    .find(|r| &self.storage[(*r).clone()] == path.as_bytes())
            })
            .cloned();
        let range = if let Some(range) = existing {
            range
        } else {
            if self.unique >= self.limits.paths {
                return Err(Error::PathLimit);
            }
            let end = self
                .storage
                .len()
                .checked_add(path.len())
                .ok_or(Error::ByteLimit)?;
            if end > self.limits.bytes {
                return Err(Error::ByteLimit);
            }
            let range = self.storage.len()..end;
            self.storage.extend_from_slice(path.as_bytes());
            self.paths.entry(hash).or_default().push(range.clone());
            self.unique += 1;
            range
        };
        let ticket = Ticket {
            epoch: self.epoch,
            row: self.rows.len(),
        };
        self.rows.push(Row {
            tenant,
            path: range,
            bytes,
        });
        self.total = total;
        Ok(ticket)
        // TASK_END push
    }
    pub fn get(&self, ticket: Ticket) -> Option<View<'_>> {
        // TASK_BEGIN get
        if ticket.epoch != self.epoch {
            return None;
        }
        let r = self.rows.get(ticket.row)?;
        Some(View {
            tenant: r.tenant,
            path: std::str::from_utf8(&self.storage[r.path.clone()])
                .expect("only UTF-8 paths inserted"),
            bytes: r.bytes,
        })
        // TASK_END get
    }
    pub fn reset(&mut self) -> Result<(), Error> {
        // TASK_BEGIN reset
        todo!("Reset logical state, invalidate tickets, and apply retention thresholds")

        // TASK_END reset
    }
}
#[cfg(test)]
mod internal {
    use super::*;
    #[test]
    fn epoch_exhaustion_preserves_batch() {
        let mut b = Batch::new(Limits::default());
        let _ = b.push(1, "/x", 2).unwrap();
        b.epoch = u64::MAX;
        let before = b.footprint();
        assert_eq!(b.reset(), Err(Error::EpochExhausted));
        assert_eq!(b.footprint(), before);
        assert_eq!(b.total_bytes(), 2);
    }
}
/// Independent per-row ownership baseline for the measurement harness.
pub fn owned_baseline(rows: &[(u32, String, u64)]) -> (Vec<(u32, String, u64)>, u64) {
    let owned = rows.to_vec();
    let total = owned.iter().map(|r| r.2).sum();
    (owned, total)
}
pub fn fixture(rows: usize, unique: usize) -> Vec<(u32, String, u64)> {
    assert!(unique > 0);
    (0..rows)
        .map(|i| {
            (
                (i % 11) as u32,
                format!("/api/items/{:08}/details", i % unique),
                (i % 4096) as u64,
            )
        })
        .collect()
}
