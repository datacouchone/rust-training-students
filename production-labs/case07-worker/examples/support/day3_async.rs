#![allow(dead_code)]
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use tokio::{runtime::{Builder, Runtime}, sync::oneshot, task::JoinHandle};
pub fn runtime() -> Runtime {
    Builder::new_current_thread().enable_time().start_paused(true).build().unwrap()
}
pub fn count(n: &Arc<AtomicUsize>) -> usize { n.load(Ordering::SeqCst) }
pub struct Slot(pub Arc<AtomicUsize>);
impl Slot {
    pub fn new(n: &Arc<AtomicUsize>) -> Self {
        n.fetch_add(1, Ordering::SeqCst);
        Self(Arc::clone(n))
    }
}
impl Drop for Slot {
    fn drop(&mut self) { self.0.fetch_sub(1, Ordering::SeqCst); }
}
struct Tracked {
    slot: Option<Slot>,
    stopped: Option<oneshot::Sender<()>>,
}
impl Drop for Tracked {
    fn drop(&mut self) {
        drop(self.slot.take());
        if let Some(tx)=self.stopped.take() { let _=tx.send(()); }
    }
}
pub fn parked(n: &Arc<AtomicUsize>) -> (JoinHandle<()>, oneshot::Receiver<()>, oneshot::Receiver<()>) {
    let n=Arc::clone(n);
    let (ready_tx, ready_rx)=oneshot::channel();
    let (stop_tx, stop_rx)=oneshot::channel();
    let handle=tokio::spawn(async move {
        let _tracked=Tracked { slot: Some(Slot::new(&n)), stopped: Some(stop_tx) };
        ready_tx.send(()).unwrap();
        std::future::pending::<()>().await;
    });
    (handle, ready_rx, stop_rx)
}
pub struct AbortOnDrop(pub JoinHandle<()>);
impl Drop for AbortOnDrop {
    fn drop(&mut self) { self.0.abort(); }
}
