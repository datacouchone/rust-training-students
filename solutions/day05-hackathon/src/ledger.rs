use crate::boundary::{Validated, bytes_equal};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Account {
    pub count: u64,
    pub bytes: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Reject {
    Conflict,
    Full,
    Overflow,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Applied,
    Replay,
}
pub struct Ledger {
    pub accounts: BTreeMap<u32, Account>,
    pub receipts: BTreeMap<(u32, u64), Vec<u8>>,
    pub max_tenants: usize,
    pub max_receipts: usize,
}
impl Ledger {
    pub fn new(max_tenants: usize, max_receipts: usize) -> Self {
        Self {
            accounts: BTreeMap::new(),
            receipts: BTreeMap::new(),
            max_tenants,
            max_receipts,
        }
    }
    pub fn apply(&mut self, e: &Validated) -> Result<Outcome, Reject> {
        let key = (e.tenant(), e.id());
        if let Some(old) = self.receipts.get(&key) {
            return if bytes_equal(old, e.body()) {
                Ok(Outcome::Replay)
            } else {
                Err(Reject::Conflict)
            };
        }
        if self.receipts.len() >= self.max_receipts
            || (!self.accounts.contains_key(&e.tenant()) && self.accounts.len() >= self.max_tenants)
        {
            return Err(Reject::Full);
        }
        let old = self.accounts.get(&e.tenant()).cloned().unwrap_or_default();
        let next = Account {
            count: old.count.checked_add(1).ok_or(Reject::Overflow)?,
            bytes: old.bytes.checked_add(e.bytes()).ok_or(Reject::Overflow)?,
        };
        self.accounts.insert(e.tenant(), next);
        self.receipts.insert(key, e.body().to_vec());
        Ok(Outcome::Applied)
    }
}
