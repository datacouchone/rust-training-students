#[derive(Clone, Copy)]
pub struct Row {
    pub valid: bool,
    pub status: u16,
    pub latency: u32,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Summary {
    pub count: u64,
    pub total: u64,
}
pub fn summarize(rows: &[Row], threshold: u32) -> Summary {
    // REPAIR: complete predicate, widen before summation, avoid materializing selected rows.
    let selected: Vec<_> = rows
        .iter()
        .filter(|r| r.status >= 500 && r.latency >= threshold)
        .collect();
    Summary {
        count: selected.len() as u64,
        total: selected.iter().map(|r| r.latency as u64).sum(),
    }
}
pub struct Buffer {
    pub bytes: Vec<u8>,
    pub retain: usize,
}
impl Buffer {
    pub fn flush(&mut self) {
        self.bytes.clear();
    }
}
