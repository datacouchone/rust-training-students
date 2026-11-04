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
    validate(c, q)?;
    let mut totals = [0u64; 4];
    let mut counts = [0u64; 4];
    let bulk = c.latency.len() / 4 * 4;
    for start in (0..bulk).step_by(4) {
        for lane in 0..4 {
            let i = start + lane;
            if selected(c, q, i) {
                counts[lane] += 1;
                totals[lane] = totals[lane]
                    .checked_add(c.latency[i] as u64)
                    .ok_or(Error::Overflow)?;
            }
        }
    }
    let mut out = Summary::default();
    for lane in 0..4 {
        out.matched += counts[lane];
        out.total_latency = out
            .total_latency
            .checked_add(totals[lane])
            .ok_or(Error::Overflow)?;
    }
    for i in bulk..c.latency.len() {
        if selected(c, q, i) {
            out.matched += 1;
            out.total_latency = out
                .total_latency
                .checked_add(c.latency[i] as u64)
                .ok_or(Error::Overflow)?;
        }
    }
    Ok(out)
    // TASK_END unrolled
}
/// Build one predicate result per lane. All four conditions must hold.
pub fn predicate_mask(latency: Simd<u32, 8>, status: Simd<u32, 8>, valid: Mask<i32, 8>, q: Query) -> Mask<i32, 8> {
    // TASK_BEGIN mask
    valid & latency.simd_ge(Simd::splat(q.min_latency)) & status.simd_ge(Simd::splat(q.min_status)) & status.simd_le(Simd::splat(599))
    // TASK_END mask
}
/// Mask out rejected lanes, then widen BEFORE reducing. Return count and total.
pub fn reduce_selected(latency: Simd<u32, 8>, mask: Mask<i32, 8>) -> Summary {
    // TASK_BEGIN reduction
    Summary { matched: mask.to_bitmask().count_ones() as u64, total_latency: mask.select(latency, Simd::splat(0)).cast::<u64>().reduce_sum() }
    // TASK_END reduction
}
pub fn scan_simd(c: Columns<'_>, q: Query) -> Result<Summary, Error> {
    validate(c, q)?;
    let mut out = Summary::default();
    let bulk = c.latency.len() / 8 * 8;
    for start in (0..bulk).step_by(8) {
        let latency = Simd::<u32, 8>::from_slice(&c.latency[start..start + 8]);
        let status = Simd::<u32, 8>::from_slice(&c.status[start..start + 8]);
        let valid = Mask::<i32, 8>::from_array(std::array::from_fn(|lane| {
            c.validity[start / 8] & (1 << lane) != 0
        }));
        let part = reduce_selected(latency, predicate_mask(latency, status, valid, q));
        out.matched += part.matched;
        out.total_latency = out.total_latency.checked_add(part.total_latency).ok_or(Error::Overflow)?;
    }
    // TASK_BEGIN tail
    // Process indices bulk..len. Validity uses the GLOBAL index, not a tail-local index.
    for i in bulk..c.latency.len() { if selected(c,q,i) { out.matched += 1; out.total_latency = out.total_latency.checked_add(c.latency[i] as u64).ok_or(Error::Overflow)?; } }
    // TASK_END tail
    Ok(out)
}
/// Start with the SIMD candidate so tests exercise your kernel. Change only with evidence.
pub fn scan_auto(c: Columns<'_>, q: Query) -> Result<Summary, Error> {
    scan_simd(c, q)
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
