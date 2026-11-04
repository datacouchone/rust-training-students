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
    // REPAIR retention: empty is not the same as releasing a burst allocation.
    pub fn flush(&mut self) {
        self.bytes.clear();
        if self.bytes.capacity() > self.retain {
            self.bytes = Vec::new();
        }
    }
}
