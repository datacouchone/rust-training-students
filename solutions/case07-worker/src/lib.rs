//! Bounded request fan-out with deadlines and explicit cancellation outcomes.
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::watch,
    task::{JoinHandle, JoinSet},
    time,
};
pub const MAX_BATCH: usize = 4096;
#[derive(Clone, Debug)]
pub struct Job {
    pub id: u64,
    pub latency: Duration,
    pub fail: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done(u64),
    Failed,
    TimedOut,
    Cancelled,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ZeroLimit,
    TooMany,
    DuplicateId,
    WorkerPanicked,
}
#[derive(Default)]
pub struct Probe {
    active: AtomicUsize,
    peak: AtomicUsize,
}
impl Probe {
    pub fn active(&self) -> usize {
        self.active.load(Ordering::SeqCst)
    }
    pub fn peak(&self) -> usize {
        self.peak.load(Ordering::SeqCst)
    }
}
struct Active(Arc<Probe>);
impl Active {
    fn new(p: Arc<Probe>) -> Self {
        let n = p.active.fetch_add(1, Ordering::SeqCst) + 1;
        p.peak.fetch_max(n, Ordering::SeqCst);
        Self(p)
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::SeqCst);
    }
}
async fn work(j: Job, p: Arc<Probe>) -> Outcome {
    let _active = Active::new(p);
    time::sleep(j.latency).await;
    if j.fail {
        Outcome::Failed
    } else {
        Outcome::Done(j.id)
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
pub async fn run_batch(
    jobs: Vec<Job>,
    limit: usize,
    deadline: Duration,
    mut cancel: watch::Receiver<bool>,
    probe: Arc<Probe>,
) -> Result<BTreeMap<u64, Outcome>, Error> {
    // TASK_BEGIN run
    if limit == 0 {
        return Err(Error::ZeroLimit);
    }
    if jobs.len() > MAX_BATCH {
        return Err(Error::TooMany);
    }
    let ids: BTreeSet<_> = jobs.iter().map(|j| j.id).collect();
    if ids.len() != jobs.len() {
        return Err(Error::DuplicateId);
    }
    let mut remaining = jobs.into_iter();
    let mut tasks = JoinSet::new();
    let mut outcomes = BTreeMap::new();
    let mut exhausted = false;
    loop {
        if *cancel.borrow() {
            tasks.abort_all();
            while tasks.join_next().await.is_some() {}
            for id in &ids {
                outcomes.entry(*id).or_insert(Outcome::Cancelled);
            }
            return Ok(outcomes);
        }
        while tasks.len() < limit && !exhausted {
            match remaining.next() {
                Some(j) => {
                    let p = probe.clone();
                    tasks.spawn(async move {
                        let id = j.id;
                        let o = time::timeout(deadline, work(j, p))
                            .await
                            .unwrap_or(Outcome::TimedOut);
                        (id, o)
                    });
                }
                None => exhausted = true,
            }
        }
        if tasks.is_empty() {
            return Ok(outcomes);
        }
        tokio::select! {biased;
         _=cancelled(&mut cancel)=>{continue;}
         result=tasks.join_next()=>{match result.unwrap(){Ok((id,o))=>{outcomes.insert(id,o);},Err(_)=>{tasks.abort_all();while tasks.join_next().await.is_some(){}return Err(Error::WorkerPanicked);}}}
        }
    }
    // TASK_END run
}
/// Owning a JoinHandle is not by itself cancellation ownership. This guard supplies it.
struct AbortOnDrop<T>(JoinHandle<T>);
impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum JoinFailure {
    Deadline,
    Task,
}
pub fn join_deadline<T>(
    handle: JoinHandle<T>,
    deadline: Duration,
) -> impl std::future::Future<Output = Result<T, JoinFailure>> {
    // TASK_BEGIN join_deadline
    let mut owned = AbortOnDrop(handle);
    async move {
        match time::timeout(deadline, &mut owned.0).await {
            Ok(r) => r.map_err(|_| JoinFailure::Task),
            Err(_) => {
                owned.0.abort();
                let _ = (&mut owned.0).await;
                Err(JoinFailure::Deadline)
            }
        }
    }
    // TASK_END join_deadline
}
