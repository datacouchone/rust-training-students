//! Bounded NDJSON serialization for a telemetry export batch.
//! One encoder per worker, consumed synchronously before reset: no shared lock.
use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
pub struct Event {
    pub tenant: u32,
    pub path: String,
    pub status: u16,
    pub latency_us: u32,
}
#[derive(Clone, Copy, Debug)]
pub enum Policy {
    Fresh,
    Reuse,
    Bounded,
}
pub struct Encoder {
    buffer: Vec<u8>,
    max_bytes: usize,
    retain_bytes: usize,
}
struct Limited<'a> {
    bytes: &'a mut Vec<u8>,
    max: usize,
}
impl Write for Limited<'_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if input.len() > self.max.saturating_sub(self.bytes.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "encoded batch quota",
            ));
        }
        self.bytes.extend_from_slice(input);
        Ok(input.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Encoder {
    pub fn new(max_bytes: usize, retain_bytes: usize) -> Result<Self, &'static str> {
        if max_bytes == 0 || retain_bytes > max_bytes {
            return Err("invalid buffer policy");
        }
        Ok(Self {
            buffer: Vec::new(),
            max_bytes,
            retain_bytes,
        })
    }
    /// Returns bytes only when the WHOLE batch encoded successfully.
    /// A caller must finish writing these borrowed bytes before the next encode.
    pub fn encode(&mut self, events: &[Event], policy: Policy) -> io::Result<&[u8]> {
        self.reset(policy);
        let result = (|| {
            let mut writer = Limited {
                bytes: &mut self.buffer,
                max: self.max_bytes,
            };
            for event in events {
                serde_json::to_writer(&mut writer, event).map_err(io::Error::other)?;
                writer.write_all(b"\n")?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            // No partial output escapes after a quota/serialization failure.
            self.reset(policy);
            return Err(error);
        }
        Ok(&self.buffer)
    }
    pub fn reset(&mut self, policy: Policy) {
        self.buffer.clear();
        if matches!(policy, Policy::Fresh)
            || (matches!(policy, Policy::Bounded) && self.buffer.capacity() > self.retain_bytes)
        {
            self.buffer = Vec::new();
        }
    }
    pub fn retained_capacity(&self) -> usize {
        self.buffer.capacity()
    }
}
pub fn fixture(rows: usize, unique_paths: usize) -> Vec<Event> {
    assert!(unique_paths > 0);
    (0..rows)
        .map(|i| Event {
            tenant: (i % 31) as u32,
            path: format!("/orders/{}/items", i % unique_paths),
            status: if i % 19 == 0 { 503 } else { 200 },
            latency_us: (i * 97 % 100_000) as u32,
        })
        .collect()
}
