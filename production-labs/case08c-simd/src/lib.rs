#![feature(portable_simd)]
//! Predicate scan over telemetry columns with a least-significant-bit-first validity bitmap.
//! Original classroom format inspired by Arrow's columnar/validity concepts, not Arrow IPC.
use std::simd::{Mask, Select, Simd, cmp::SimdPartialOrd, num::SimdUint};
#[derive(Clone, Copy)]
pub struct Columns<'a> {
    pub latency: &'a [u32],
    pub status: &'a [u32],
    pub validity: &'a [u8],
}
#[derive(Clone, Copy)]
pub struct Query {
    pub min_latency: u32,
    pub min_status: u32,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub matched: u64,
    pub total_latency: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Length,
    Bitmap,
    Query,
    Overflow,
}
pub fn validate(c: Columns<'_>, q: Query) -> Result<(), Error> {
    if c.latency.len() != c.status.len() {
        return Err(Error::Length);
    }
    if c.validity.len() != c.latency.len().div_ceil(8) {
        return Err(Error::Bitmap);
    }
    if !(100..=599).contains(&q.min_status) {
        return Err(Error::Query);
    }
    Ok(())
}
fn selected(c: Columns<'_>, q: Query, i: usize) -> bool {
    c.validity[i / 8] & (1 << (i % 8)) != 0
        && c.status[i] >= q.min_status
        && c.status[i] <= 599
        && c.latency[i] >= q.min_latency
}
/// Independent readable oracle, also the fair scalar measurement baseline.
pub fn scan_scalar(c: Columns<'_>, q: Query) -> Result<Summary, Error> {
    validate(c, q)?;
    let mut out = Summary::default();
    for i in 0..c.latency.len() {
        if selected(c, q, i) {
            out.matched += 1;
            out.total_latency = out
                .total_latency
                .checked_add(c.latency[i] as u64)
                .ok_or(Error::Overflow)?;
        }
    }
    Ok(out)
}
pub fn scan_unrolled(c: Columns<'_>, q: Query) -> Result<Summary, Error> {
    // TASK_BEGIN unrolled
    todo!("Implement unrolled: see README contract and acceptance tests");
    // TASK_END unrolled
}
pub fn scan_simd(c: Columns<'_>, q: Query) -> Result<Summary, Error> {
    // TASK_BEGIN simd
    todo!("Implement simd: see README contract and acceptance tests");
    // TASK_END simd
}
/// Choose a threshold from measurements. Correctness is mandatory for either branch.
pub fn scan_auto(c: Columns<'_>, q: Query) -> Result<Summary, Error> {
    // TASK_BEGIN dispatch
    todo!("Implement dispatch: see README contract and acceptance tests");
    // TASK_END dispatch
}
pub struct OwnedColumns {
    pub latency: Vec<u32>,
    pub status: Vec<u32>,
    pub validity: Vec<u8>,
}
impl OwnedColumns {
    pub fn view(&self) -> Columns<'_> {
        Columns {
            latency: &self.latency,
            status: &self.status,
            validity: &self.validity,
        }
    }
}
pub fn fixture(n: usize, sparse: bool) -> OwnedColumns {
    let latency = (0..n)
        .map(|i| ((i as u64 * 7919) % 100_000) as u32)
        .collect();
    let status = (0..n)
        .map(|i| if i % 19 == 0 { 503 } else { 200 })
        .collect();
    let validity = vec![if sparse { 0b00010001 } else { 255 }; n.div_ceil(8)];
    OwnedColumns {
        latency,
        status,
        validity,
    }
}
