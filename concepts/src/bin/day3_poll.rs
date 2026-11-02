fn main() {
    // TASK_BEGIN demonstrate
    let signal = Arc::new(Signal(AtomicBool::new(false)));
    let waker = Waker::from(signal.clone());
    let mut context = Context::from_waker(&waker);
    let mut request = Box::pin(Response(false));
    let first = request.as_mut().poll(&mut context);
    println!("first poll: {first:?}");
    println!("notification waiting: {}", signal.0.load(Ordering::Relaxed));
    println!("driver stopped before the next poll");
    // TASK_END demonstrate
}

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::task::{Context, Poll, Wake, Waker};
struct Signal(AtomicBool);
impl Wake for Signal {
    fn wake(self: Arc<Self>) { self.0.store(true, Ordering::Relaxed); }
}
struct Response(bool);
impl Future for Response {
    type Output = &'static str;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 { return Poll::Ready("response"); }
        self.0 = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
