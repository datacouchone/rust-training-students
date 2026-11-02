//! Route-policy lookup on an edge gateway. Original teaching model; not Pingora code.
use std::collections::{BTreeMap, HashMap};
#[derive(Clone, Debug)]
pub struct Rule {
    pub tenant: Option<u32>,
    pub prefix: String,
    pub pool: u16,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidPrefix,
    Duplicate,
}
#[derive(Default)]
struct Destinations {
    tenants: HashMap<u32, u16>,
    fallback: Option<u16>,
}
pub struct Index {
    entries: HashMap<String, Destinations>,
}
fn valid_path(s: &str) -> bool {
    s.starts_with('/') && !s.contains(['?', '#']) && !s.contains("//")
}
fn matches(prefix: &str, path: &str) -> bool {
    prefix == "/"
        || path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.starts_with('/'))
}
/// Supplied scan baseline. Longest segment prefix wins; tenant-specific wins a same-prefix tie.
#[inline(never)]
pub fn linear(rules: &[Rule], tenant: u32, path: &str) -> Option<u16> {
    if !valid_path(path) {
        return None;
    }
    let mut best: Option<(usize, bool, u16)> = None;
    for r in rules {
        if r.tenant.is_some_and(|t| t != tenant) || !matches(&r.prefix, path) {
            continue;
        }
        let rank = (r.prefix.len(), r.tenant.is_some());
        if best.is_none_or(|(len, specific, _)| rank > (len, specific)) {
            best = Some((rank.0, rank.1, r.pool));
        }
    }
    best.map(|(_, _, pool)| pool)
}
impl Index {
    pub fn compile(rules: &[Rule]) -> Result<Self, Error> {
        // TASK_BEGIN compile
    todo!("Implement compile: see README contract and acceptance tests");
        // TASK_END compile
    }
    /// The index owns a policy snapshot; later edits to the input do not mutate it.
    #[inline(never)]
    pub fn route(&self, tenant: u32, path: &str) -> Option<u16> {
        // TASK_BEGIN route
    todo!("Implement route: see README contract and acceptance tests");
        // TASK_END route
    }
}
#[derive(Debug)]
pub struct Request<'a> {
    pub tenant: u32,
    pub path: &'a str,
}
#[inline(never)]
pub fn parse_requests(text: &str) -> Vec<Request<'_>> {
    text.lines()
        .filter_map(|line| {
            let (t, path) = line.split_once('\t')?;
            Some(Request {
                tenant: t.parse().ok()?,
                path,
            })
        })
        .collect()
}
#[inline(never)]
pub fn scan_requests(rules: &[Rule], requests: &[Request<'_>]) -> Vec<Option<u16>> {
    requests
        .iter()
        .map(|r| linear(rules, r.tenant, r.path))
        .collect()
}
#[inline(never)]
pub fn indexed_requests(index: &Index, requests: &[Request<'_>]) -> Vec<Option<u16>> {
    requests
        .iter()
        .map(|r| index.route(r.tenant, r.path))
        .collect()
}
#[inline(never)]
pub fn summarize(results: &[Option<u16>]) -> BTreeMap<Option<u16>, usize> {
    let mut out = BTreeMap::new();
    for pool in results {
        *out.entry(*pool).or_default() += 1;
    }
    out
}
/// Fixed policy/workload generation. Inputs are synthetic, deterministic and credential-free.
pub fn fixture(rows: usize, routes: usize) -> (Vec<Rule>, String) {
    assert!(routes > 0);
    let mut policy = vec![Rule {
        tenant: None,
        prefix: "/".into(),
        pool: 0,
    }];
    for i in 0..routes {
        policy.push(Rule {
            tenant: None,
            prefix: format!("/api/{i}"),
            pool: (i % 32 + 1) as u16,
        });
        if i % 7 == 0 {
            policy.push(Rule {
                tenant: Some(7),
                prefix: format!("/api/{i}"),
                pool: 63,
            });
        }
    }
    let mut input = String::new();
    use std::fmt::Write;
    for i in 0..rows {
        writeln!(
            input,
            "{}\t/api/{}/item/{}",
            i % 11,
            (i * 7919) % routes,
            i % 29
        )
        .unwrap();
    }
    (policy, input)
}
