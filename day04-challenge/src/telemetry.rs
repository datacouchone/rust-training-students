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
// REPAIR query semantics before tuning it. Keep valid server errors >= threshold.
pub fn summarize(events: &[Event], threshold: u32) -> Summary {
    let selected: Vec<_> = events
        .iter()
        .filter(|e| e.latency >= threshold && e.status >= 500)
        .collect();
    Summary {
        count: selected.len() as u64,
        total: selected.iter().map(|e| e.latency as u64).sum(),
    }
}
pub struct Buffer {
    pub bytes: Vec<u8>,
    pub retain: usize,
}
impl Buffer {
    // REPAIR retention: empty is not the same as releasing a burst allocation.
    pub fn flush(&mut self) {
        self.bytes.clear();
    }
}
