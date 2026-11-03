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
        // REPAIR: same identity + same full body is replay; changed body is conflict.
        if let Some(old) = self.receipts.get(&key)
            && !bytes_equal(old, e.body())
        {
            return Err(Reject::Conflict);
        }
        // REPAIR: both cardinalities and arithmetic are checked before any mutation.
        let a = self.accounts.entry(e.tenant()).or_default();
        a.count = a.count.checked_add(1).ok_or(Reject::Overflow)?;
        a.bytes = a.bytes.checked_add(e.bytes()).ok_or(Reject::Overflow)?;
        self.receipts.insert(key, e.body().to_vec());
        Ok(Outcome::Applied)
    }
}
