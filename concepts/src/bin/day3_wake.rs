fn main() {
    // TASK_BEGIN demonstrate
    let queue = Arc::new(ReadyFlag(AtomicBool::new(false)));
    let waker = Waker::from(queue.clone());
    waker.wake_by_ref();
    waker.wake_by_ref();
    println!("notifications sent: 2");
    println!("queued turns: {}", usize::from(queue.0.load(Ordering::Relaxed)));
    // TASK_END demonstrate
}

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::task::{Wake, Waker};
struct ReadyFlag(AtomicBool);
impl Wake for ReadyFlag {
    fn wake(self: Arc<Self>) { self.0.store(true, Ordering::Relaxed); }
}
