use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
#[derive(Debug)]
pub struct Ledger {
    pub used: usize,
    pub accepted: usize,
    pub limit: usize,
}
pub struct Lease {
    state: Arc<Mutex<Ledger>>,
    bytes: usize,
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.state.lock().unwrap().used -= self.bytes;
    }
}
// REPAIR admission: rejected work must not alter used or accepted; no wrapping.
pub fn admit(state: &Arc<Mutex<Ledger>>, bytes: usize) -> Option<Lease> {
    let mut s = state.lock().unwrap();
    let next = s.used.checked_add(bytes)?;
    let accepted = s.accepted.checked_add(1)?;
    if next > s.limit {
        return None;
    }
    s.used = next;
    s.accepted = accepted;
    Some(Lease {
        state: Arc::clone(state),
        bytes,
    })
}
// REPAIR retry budget: every successful take consumes exactly one available token.
pub fn take_retry(tokens: &AtomicUsize) -> bool {
    let mut current = tokens.load(Ordering::Relaxed);
    loop {
        let Some(next) = current.checked_sub(1) else {
            return false;
        };
        match tokens.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return true,
            Err(actual) => current = actual,
        }
    }
}
#[derive(Clone)]
pub struct Rule {
    pub tenant: u32,
    pub prefix: String,
    pub pool: u16,
}
pub struct Router {
    pub rules: Vec<Rule>,
    pub index: HashMap<String, HashMap<u32, u16>>,
    pub inspected: AtomicUsize,
}
impl Router {
    pub fn new(extra: usize) -> Self {
        let mut rules = vec![
            Rule {
                tenant: 0,
                prefix: "/".into(),
                pool: 0,
            },
            Rule {
                tenant: 0,
                prefix: "/api".into(),
                pool: 1,
            },
            Rule {
                tenant: 7,
                prefix: "/api/orders".into(),
                pool: 17,
            },
        ];
        for i in 0..extra {
            rules.push(Rule {
                tenant: 0,
                prefix: format!("/unused/{i}"),
                pool: 2,
            });
        }
        let mut index: HashMap<String, HashMap<u32, u16>> = HashMap::new();
        for r in &rules {
            index
                .entry(r.prefix.clone())
                .or_default()
                .insert(r.tenant, r.pool);
        }
        Self {
            rules,
            index,
            inspected: AtomicUsize::new(0),
        }
    }
    // REPAIR routing cost: preserve longest segment prefix, tenant breaks ties.
    // The index is built once above; count examined prefixes in inspected.
    pub fn route(&self, tenant: u32, path: &str) -> Option<u16> {
        if !path.starts_with('/') {
            return None;
        }
        let mut prefix = path;
        loop {
            self.inspected.fetch_add(1, Ordering::Relaxed);
            if let Some(pool) = self
                .index
                .get(prefix)
                .and_then(|tenants| tenants.get(&tenant).or_else(|| tenants.get(&0)))
            {
                return Some(*pool);
            }
            if prefix == "/" {
                return None;
            }
            let end = prefix.rfind('/')?;
            prefix = if end == 0 { "/" } else { &prefix[..end] };
        }
    }
}
