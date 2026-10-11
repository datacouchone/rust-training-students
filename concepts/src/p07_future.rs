use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
// Toy scheduling examples, not wall-clock timers. Both structs are Unpin.
pub struct Countdown {
    pub remaining: u8,
}
impl Future for Countdown {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // PUZZLE BEGIN demo
        todo!("p07_future: demo")
        // PUZZLE END demo
    }
}
pub struct AddLater {
    pub a: i32,
    pub b: i32,
    pub waited: bool,
}
impl Future for AddLater {
    type Output = i32;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
        // PUZZLE BEGIN exercise
        todo!("p07_future: exercise")
        // PUZZLE END exercise
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::task::{Wake, Waker};
    struct Counter(AtomicUsize);
    impl Wake for Counter {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    #[test]
    fn demo_pending_wakes_then_ready() {
        let counter = Arc::new(Counter(AtomicUsize::new(0)));
        let waker = Waker::from(counter.clone());
        let mut cx = Context::from_waker(&waker);
        let mut future = Box::pin(Countdown { remaining: 2 });
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Ready(()));
        assert_eq!(counter.0.load(Ordering::Relaxed), 2);
    }
    #[test]
    fn exercise_one_pending_and_sum() {
        let counter = Arc::new(Counter(AtomicUsize::new(0)));
        let waker = Waker::from(counter.clone());
        let mut cx = Context::from_waker(&waker);
        let mut future = Box::pin(AddLater {
            a: 4,
            b: -1,
            waited: false,
        });
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(counter.0.load(Ordering::Relaxed), 1);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Ready(3));
    }
}
