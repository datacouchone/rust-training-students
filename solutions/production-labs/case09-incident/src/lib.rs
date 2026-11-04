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
        // TASK_BEGIN apply
        let text = std::str::from_utf8(body).map_err(|_| Reject::Encoding)?;
        let (tenant, bytes) = text.split_once(':').ok_or(Reject::Shape)?;
        let tenant = tenant.parse::<u32>().map_err(|_| Reject::Number)?;
        let bytes = bytes.parse::<u64>().map_err(|_| Reject::Number)?;
        if !self.accounts.contains_key(&tenant) && self.accounts.len() >= self.max_tenants {
            return Err(Reject::Cardinality);
        }
        let old = self.accounts.get(&tenant).cloned().unwrap_or_default();
        let new = Account {
            accepted: old.accepted.checked_add(1).ok_or(Reject::Overflow)?,
            bytes: old.bytes.checked_add(bytes).ok_or(Reject::Overflow)?,
        };
        self.accounts.insert(tenant, new.clone());
        Ok(new)
        // TASK_END apply
    }
}
pub fn frame(body: &[u8]) -> Vec<u8> {
    let mut b = (body.len() as u32).to_be_bytes().to_vec();
    b.extend(body);
    b
}
