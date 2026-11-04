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
    let mut out = Summary { count: 0, total: 0 };
    for r in rows {
        if r.valid && (500..=599).contains(&r.status) && r.latency >= threshold {
            out.count += 1;
            out.total += u64::from(r.latency);
        }
    }
    out
}

pub struct Buffer {
    pub bytes: Vec<u8>,
    pub retain: usize,
}
impl Buffer {
    pub fn flush(&mut self) {
        if self.bytes.capacity() > self.retain {
            self.bytes = Vec::new()
        } else {
            self.bytes.clear()
        }
    }
}
