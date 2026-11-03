#![allow(dead_code)]
//! Stateful frame reader and transactional tenant accounting for a telemetry gateway.
//! A cancelled read must preserve already consumed bytes. A terminal protocol error poisons the stream.
use std::{collections::BTreeMap, io};
use tokio::io::{AsyncRead, AsyncReadExt};
pub const MAX_FRAME: usize = 4096;
pub struct Decoder {
    header: [u8; 4],
    header_read: usize,
    body: Vec<u8>,
    body_read: usize,
    length: Option<usize>,
    poisoned: bool,
}
impl Default for Decoder {
    fn default() -> Self {
        Self {
            header: [0; 4],
            header_read: 0,
            body: vec![],
            body_read: 0,
            length: None,
            poisoned: false,
        }
    }
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
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Account {
    pub accepted: u64,
    pub bytes: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Reject {
    Encoding,
    Shape,
    Number,
    Overflow,
    Cardinality,
}
pub struct Gateway {
    accounts: BTreeMap<u32, Account>,
    max_tenants: usize,
}
impl Gateway {
    pub fn new(max_tenants: usize) -> Self {
        Self {
            accounts: BTreeMap::new(),
            max_tenants,
        }
    }
    pub fn accounts(&self) -> &BTreeMap<u32, Account> {
        &self.accounts
    }
    /// Body is tenant_id:byte_count. Reject without changing any account.
    pub fn apply(&mut self, body: &[u8]) -> Result<Account, Reject> {
        let text = std::str::from_utf8(body).map_err(|_| Reject::Encoding)?;
        let (tenant, bytes) = text.split_once(':').ok_or(Reject::Shape)?;
        let tenant = tenant.parse::<u32>().map_err(|_| Reject::Number)?;
        let bytes = bytes.parse::<u64>().map_err(|_| Reject::Number)?;
        let account = self.accounts.entry(tenant).or_default();
        account.accepted = account.accepted.checked_add(1).ok_or(Reject::Overflow)?;
        account.bytes = account.bytes.checked_add(bytes).ok_or(Reject::Overflow)?;
        Ok(account.clone())
    }
}
pub fn frame(body: &[u8]) -> Vec<u8> {
    let mut b = (body.len() as u32).to_be_bytes().to_vec();
    b.extend(body);
    b
}
