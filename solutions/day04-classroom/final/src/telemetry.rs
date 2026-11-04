#[derive(Clone, Copy)]
pub struct Event {
    pub latency: u32,
    pub status: u16,
    pub valid: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Summary {
    pub count: u64,
    pub total: u64,
}
// Independent scalar oracle: preserve it for differential tests and fair timing.
pub fn summarize(events: &[Event], threshold: u32) -> Summary {
    events
        .iter()
        .filter(|e| e.valid && e.latency >= threshold && (500..=599).contains(&e.status))
        .fold(Summary { count: 0, total: 0 }, |mut out, e| {
            out.count += 1;
            out.total += u64::from(e.latency);
            out
        })
}
pub struct Buffer {
    pub bytes: Vec<u8>,
    pub retain: usize,
}
impl Buffer {
    // Supplied non-SIMD repair: clear logical state and bound retained capacity.
    pub fn flush(&mut self) {
        self.bytes.clear();
        if self.bytes.capacity() > self.retain {
            self.bytes = Vec::new();
        }
    }
}

/// Transfer task: events are rows (AoS), unlike 8C's contiguous columns (SoA).
/// Packing each field has a cost. Include it when comparing against summarize.
pub fn summarize_simd(events: &[Event], threshold: u32) -> Summary {
    use std::simd::{Mask, Select, Simd, cmp::SimdPartialOrd, num::SimdUint};
    let mut out = Summary { count: 0, total: 0 };
    let bulk = events.len() / 8 * 8;
    for start in (0..bulk).step_by(8) {
        // These are safe scalar field reads assembled into vectors; not contiguous column loads.
        let latency = Simd::<u32, 8>::from_array(std::array::from_fn(|lane| events[start + lane].latency));
        let status = Simd::<u32, 8>::from_array(std::array::from_fn(|lane| u32::from(events[start + lane].status)));
        let valid = Mask::<i32, 8>::from_array(std::array::from_fn(|lane| events[start + lane].valid));
        // TASK_BEGIN vector_kernel
        // Implement the predicate, count and widened sum. Keep the scalar oracle unchanged.
        let mask = valid & latency.simd_ge(Simd::splat(threshold)) & status.simd_ge(Simd::splat(500)) & status.simd_le(Simd::splat(599));
        out.count += mask.to_bitmask().count_ones() as u64;
        out.total += mask.select(latency, Simd::splat(0)).cast::<u64>().reduce_sum();
        // TASK_END vector_kernel
    }
    // TASK_BEGIN tail
    for e in &events[bulk..] { if e.valid && e.latency >= threshold && (500..=599).contains(&e.status) { out.count += 1; out.total += u64::from(e.latency); } }
    // TASK_END tail
    out
}
