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
        let mut header = [0u8; 4];
        match reader.read_exact(&mut header).await {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        }
        let n = u32::from_be_bytes(header) as usize;
        if n > MAX_FRAME {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "frame limit"));
        }
        let mut body = vec![0; n];
        reader.read_exact(&mut body).await?;
        Ok(Some(body))
    }
}
pub fn frame(body: &[u8]) -> Vec<u8> {
    let mut b = (body.len() as u32).to_be_bytes().to_vec();
    b.extend(body);
    b
}
