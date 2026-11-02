//! Original upstream-retry challenge. The scripted transport models commit uncertainty without a network.
use std::{
    collections::{HashSet, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::watch,
    time::{self, Instant},
};
#[derive(Debug, Clone)]
pub struct Operation {
    pub idempotency_key: Option<String>,
}
#[derive(Clone, Copy)]
pub struct Policy {
    pub max_attempts: usize,
    pub total_timeout: Duration,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
    /// Supplied deterministic sampler for tests. A deployment must supply suitable independent randomness.
    pub jitter: fn(Duration) -> Duration,
}
impl Policy {
    pub fn validate(self) -> bool {
        (1..=8).contains(&self.max_attempts)
            && !self.total_timeout.is_zero()
            && self.base_backoff <= self.max_backoff
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    InvalidPolicy,
    Cancelled,
    Deadline,
    Permanent,
    Uncertain,
    AttemptsExhausted,
    RetryBudget,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Failure {
    pub reason: Stop,
    pub attempts: usize,
    pub may_have_committed: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Receipt {
    pub attempts: usize,
}
#[derive(Debug, Clone, Copy)]
pub enum Response {
    Success,
    RetryableBeforeCommit { retry_after: Duration },
    CommitThenLoseReply,
    Permanent,
}
#[derive(Debug, Clone, Copy)]
pub struct Plan {
    pub after: Duration,
    pub response: Response,
}
pub struct RetryBudget {
    tokens: AtomicUsize,
}
impl RetryBudget {
    pub fn new(tokens: usize) -> Arc<Self> {
        Arc::new(Self {
            tokens: AtomicUsize::new(tokens),
        })
    }
    pub fn remaining(&self) -> usize {
        self.tokens.load(Ordering::Relaxed)
    }
    fn take(&self) -> bool {
        self.tokens
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1))
            .is_ok()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub started: usize,
    pub active: usize,
    pub commits: usize,
    pub keys: Vec<Option<String>>,
}
#[derive(Default)]
struct State {
    plans: VecDeque<Plan>,
    observation: ObservationInner,
    committed_keys: HashSet<String>,
}
#[derive(Default)]
struct ObservationInner {
    started: usize,
    active: usize,
    commits: usize,
    keys: Vec<Option<String>>,
}
#[derive(Clone)]
pub struct Scripted {
    state: Arc<Mutex<State>>,
}
struct Active(Arc<Mutex<State>>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.lock().unwrap().observation.active -= 1;
    }
}
impl Scripted {
    pub fn new(plans: Vec<Plan>) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                plans: plans.into(),
                ..State::default()
            })),
        }
    }
    pub fn observe(&self) -> Observation {
        let s = self.state.lock().unwrap();
        let o = &s.observation;
        Observation {
            started: o.started,
            active: o.active,
            commits: o.commits,
            keys: o.keys.clone(),
        }
    }
    /// The provided transport is cooperative. Dropping this future releases its active guard.
    pub async fn send(&self, key: Option<&str>) -> Response {
        let (plan, _active) = {
            let mut s = self.state.lock().unwrap();
            s.observation.started += 1;
            s.observation.active += 1;
            s.observation.keys.push(key.map(str::to_owned));
            (
                s.plans.pop_front().unwrap_or(Plan {
                    after: Duration::ZERO,
                    response: Response::Permanent,
                }),
                Active(self.state.clone()),
            )
        };
        time::sleep(plan.after).await;
        if matches!(
            plan.response,
            Response::Success | Response::CommitThenLoseReply
        ) {
            let mut s = self.state.lock().unwrap();
            let fresh = key.is_none_or(|k| s.committed_keys.insert(k.to_owned()));
            if fresh {
                s.observation.commits += 1;
            }
        }
        plan.response
    }
}
async fn cancelled(rx: &mut watch::Receiver<bool>) {
    loop {
        if *rx.borrow() {
            return;
        }
        if rx.changed().await.is_err() {
            std::future::pending::<()>().await;
        }
    }
}
/// `completed` counts completed attempts and is at least one. The sampler is clamped to the cap.
pub fn retry_delay(policy: Policy, completed: usize, retry_after: Duration) -> Duration {
    // TASK_BEGIN delay
    todo!("Implement delay: see README contract and acceptance tests");
    // TASK_END delay
}
/// One absolute deadline covers all attempts and backoff. Each extra attempt consumes one shared token.
pub async fn execute(
    operation: Operation,
    policy: Policy,
    budget: Arc<RetryBudget>,
    transport: Scripted,
    mut cancel: watch::Receiver<bool>,
) -> Result<Receipt, Failure> {
    // TASK_BEGIN execute
    todo!("Implement execute: see README contract and acceptance tests");
    // TASK_END execute
}
