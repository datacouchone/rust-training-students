#![allow(dead_code)]
//! Stateful frame reader and transactional tenant accounting for a telemetry gateway.
//! A cancelled read must preserve already consumed bytes. A terminal protocol error poisons the stream.
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt};
pub const MAX_FRAME: usize = 4096;
#[derive(Default)]
pub struct Decoder {
    header: [u8; 4],
    header_read: usize,
    body: Vec<u8>,
    body_read: usize,
    length: Option<usize>,
    poisoned: bool,
}
impl Decoder {
    pub fn buffered(&self) -> usize {
        self.header_read + self.body_read
    }
    pub async fn next<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut R,
    ) -> io::Result<Option<Vec<u8>>> {
        // TASK_BEGIN next
        if self.poisoned {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stream poisoned",
            ));
        }
        while self.header_read < 4 {
            let n = match reader.read(&mut self.header[self.header_read..]).await {
                Ok(n) => n,
                Err(e) => {
                    self.poisoned = true;
                    return Err(e);
                }
            };
            if n == 0 {
                if self.header_read == 0 {
                    return Ok(None);
                }
                self.poisoned = true;
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "partial header",
                ));
            }
            self.header_read += n;
        }
        if self.length.is_none() {
            let n = u32::from_be_bytes(self.header) as usize;
            if n > MAX_FRAME {
                self.poisoned = true;
                return Err(io::Error::new(io::ErrorKind::InvalidData, "frame limit"));
            }
            self.body.resize(n, 0);
            self.length = Some(n);
        }
        while self.body_read < self.body.len() {
            let n = match reader.read(&mut self.body[self.body_read..]).await {
                Ok(n) => n,
                Err(e) => {
                    self.poisoned = true;
                    return Err(e);
                }
            };
            if n == 0 {
                self.poisoned = true;
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "partial body"));
            }
            self.body_read += n;
        }
        let out = std::mem::take(&mut self.body);
        self.header_read = 0;
        self.body_read = 0;
        self.length = None;
        Ok(Some(out))
        // TASK_END next
    }
}
pub fn frame(body: &[u8]) -> Vec<u8> {
    let mut b = (body.len() as u32).to_be_bytes().to_vec();
    b.extend(body);
    b
}
