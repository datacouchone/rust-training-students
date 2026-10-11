//! Lab 7: a single-threaded async executor built from `std` alone.
//!
//! This crate has three pieces:
//!
//! 1. **Provided scaffolding** — `Task`, `Spawner`, `Executor`,
//!    `new_executor_and_spawner`, and `block_on`. This is the same code
//!    in `provided scaffold and the reference implementation`; it is the "harness" the rest of the
//!    lab plugs into. Read it first — everything else calls into it.
//! 2. **`TimerFuture`** (STUDENT TODO (a)) — a `Future` that completes
//!    after a duration, driven by a background thread instead of a
//!    polling loop.
//! 3. **`join_two`** (STUDENT TODO (b)) — a hand-rolled "await two
//!    futures concurrently" combinator, and the `RawWaker`/
//!    `RawWakerVTable` functions (STUDENT TODO (c)) that make `wake()`
//!    actually reschedule a task.
//!
//! Background reading: Module 7 slides on the `Future` trait, the
//! `Waker`/`Context` API, and `Pin` (self-referential futures, why
//! `poll` takes `Pin<&mut Self>`, what `Unpin` means). This file's
//! `join_two` section documents the specific `Pin` decision it makes;
//! read that alongside the `Pin` slide before starting part (b).
//!
//! Run `cargo test` as you fill in each `todo!()` — the four tests at
//! the bottom of this file are your feedback loop. All four should go
//! green once (a), (b), and (c) below are done; nothing else needs to
//! change.

use std::future::Future;
use std::mem::ManuallyDrop;
use std::panic::{self, AssertUnwindSafe};
use std::pin::Pin;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;
use std::time::Duration;

// =====================================================================
// PROVIDED SCAFFOLDING — task queue, `Executor`/`Spawner`, `block_on`.
// Identical in `provided scaffold and the reference implementation`.
// =====================================================================

/// A unit of work the executor knows how to run: a boxed, pinned future
/// plus a way to put itself back on the ready queue when woken.
struct Task {
    /// The future being driven. `None` only while a poll of it is
    /// logically "checked out" (see `Executor::poll_task`); wrapped in
    /// a `Mutex` because a `Waker` can call back into the queue from a
    /// background thread (e.g. `TimerFuture`'s sleeper thread) while
    /// this executor is (or isn't) polling.
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>,
    /// Clone of the channel back to the ready queue. The waker vtable
    /// functions (STUDENT TODO (c)) use this to resubmit `self` when
    /// woken.
    task_sender: Sender<Arc<Task>>,
}

/// Handle used to push new futures onto an executor's ready queue.
#[derive(Clone)]
pub struct Spawner {
    task_sender: Sender<Arc<Task>>,
}

/// Pulls tasks off the ready queue and polls them, single-threaded.
pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// Build a fresh executor + spawner pair sharing one queue.
///
/// The channel is unbounded: a waker firing from a background thread
/// (e.g. a `TimerFuture`'s sleeper thread) must never block on send, or
/// a slow/stuck executor could deadlock a timer thread.
pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    let (task_sender, ready_queue) = channel();
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
    /// Box, pin, and enqueue `future` for the executor to drive.
    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender
            .send(task)
            .expect("executor's receiver was dropped while spawning");
    }
}

impl Executor {
    /// Poll one task once. If it returns `Pending`, the future is put
    /// back so a future `wake()` (via the STUDENT TODO (c) vtable) can
    /// resubmit this same `Arc<Task>` to `ready_queue`; the caller does
    /// not loop on this task itself.
    fn poll_task(task: &Arc<Task>) {
        let mut slot = task.future.lock().unwrap();
        // `take()` so a poll never re-enters the same future twice
        // concurrently (e.g. if this task is somehow queued twice); if
        // the slot is already empty (already completed / mid-poll on
        // another thread) this is a harmless no-op.
        if let Some(mut future) = slot.take() {
            let waker = task_waker(task);

            // Catch a panicking poll here instead of letting it unwind
            // straight through this frame. Why: `waker` (built from the
            // STUDENT TODO (c) vtable) is still a live local at that
            // point, and a panic inside a `Drop` impl that fires *while
            // another panic is already unwinding* is unconditionally
            // fatal — Rust aborts the whole process rather than
            // double-unwind. Without this, a `todo!()` in (a) or (b)
            // would abort the entire test binary the moment `waker`'s
            // drop (routed through the still-unfinished vtable) ran
            // during that unwind, instead of failing just the one
            // test. Catching here, dropping `future`/`waker` through
            // ordinary (non-unwinding) control flow, and then
            // re-raising keeps every `todo!()` in this lab a normal,
            // separately-reported test failure no matter which of the
            // three is still unfinished.
            let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
                let mut cx = Context::from_waker(&waker);
                future.as_mut().poll(&mut cx)
            }));

            match outcome {
                Ok(Poll::Pending) => *slot = Some(future),
                Ok(Poll::Ready(())) => { /* done: drop the future, Task drops with the Arc */ }
                Err(payload) => {
                    drop(future);
                    // `waker`'s `Drop` routes through the (possibly still
                    // unfinished) `drop_raw` vtable fn (STUDENT TODO (c)),
                    // which can itself panic — e.g. it's a `todo!()` in
                    // the untouched starter. `payload` is already an
                    // in-flight panic at this point, so letting a second
                    // one escape here would be a double-panic, which
                    // Rust makes unconditionally fatal (SIGABRT) instead
                    // of a normal test failure — and it would report
                    // `drop_raw`'s message, masking the original bug.
                    // Catch and discard only this secondary panic so
                    // `payload`, the real failure, is what actually gets
                    // reported below.
                    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| drop(waker)));
                    panic::resume_unwind(payload);
                }
            }
        }
    }

    /// Poll every currently-queued task once each, then return without
    /// waiting for more work. Does not block. Useful for tests that
    /// want to single-step the scheduler, and used by `block_on`
    /// indirectly through `run`.
    pub fn run_until_stalled(&self) {
        loop {
            match self.ready_queue.try_recv() {
                Ok(task) => Self::poll_task(&task),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => return,
            }
        }
    }

    /// Drive the executor until the channel disconnects, i.e. every
    /// `Spawner`/waker clone referencing this queue has been dropped
    /// and no task is left outstanding.
    ///
    /// `Receiver::recv` blocks the calling thread on the channel's
    /// internal synchronization (parking), not a spin loop — an idle
    /// executor with pending-but-not-yet-woken tasks costs no CPU.
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            Self::poll_task(&task);
        }
    }
}

/// Block the current thread until `future` resolves, running a private
/// executor locally, and return its output.
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let (executor, spawner) = new_executor_and_spawner();
    let output: Arc<Mutex<Option<F::Output>>> = Arc::new(Mutex::new(None));
    let output_slot = Arc::clone(&output);

    spawner.spawn(async move {
        let value = future.await;
        *output_slot.lock().unwrap() = Some(value);
    });
    // Drop our clone of the sender now. Once the top-level task above
    // finishes (dropping its own clone, held in `Task::task_sender`)
    // and nothing else is holding a reference into the queue, the
    // channel disconnects and `run()` returns instead of blocking in
    // `recv` forever.
    drop(spawner);

    executor.run();

    Arc::try_unwrap(output)
        .unwrap_or_else(|_| panic!("executor stopped with an outstanding output reference"))
        .into_inner()
        .unwrap()
        .expect("top-level future did not resolve to a value")
}

// =====================================================================
// STUDENT TODO (a): `TimerFuture`
//
// A future that becomes `Ready(())` once `duration` has elapsed. It
// must NOT busy-poll: a background thread sleeps for `duration`, then
// flips shared state and calls the stored `Waker` so the executor
// re-polls exactly once, at (approximately) the right time.
//
// `TimerFuture::new` (given, below) already spawns that background
// thread and wires up `TimerShared`. Your job is `poll`:
//   - if `shared.completed` is already `true`, return `Poll::Ready(())`.
//   - otherwise, stash `cx.waker().clone()` into `shared.waker` (so the
//     background thread has something to call `.wake()` on) and return
//     `Poll::Pending`.
// =====================================================================

struct TimerShared {
    completed: bool,
    waker: Option<Waker>,
}

/// A future that resolves after roughly `duration`.
pub struct TimerFuture {
    shared: Arc<Mutex<TimerShared>>,
}

impl TimerFuture {
    /// Spawn a background thread that sleeps for `duration`, then marks
    /// the timer complete and wakes whoever is (or later is) polling
    /// it.
    pub fn new(duration: Duration) -> Self {
        let shared = Arc::new(Mutex::new(TimerShared {
            completed: false,
            waker: None,
        }));

        let thread_shared = Arc::clone(&shared);
        thread::spawn(move || {
            thread::sleep(duration);
            let mut guard = thread_shared.lock().unwrap();
            guard.completed = true;
            // Wake whichever task's poll last stored a waker here. If
            // no one has polled yet (unlikely, but possible under
            // scheduling jank), there's nothing to wake — the next
            // `poll` will see `completed == true` immediately and
            // return `Ready` without ever needing a wake-up.
            if let Some(waker) = guard.waker.take() {
                waker.wake();
            }
        });

        TimerFuture { shared }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        todo!(
            "STUDENT (a): lock `self.shared`; if `completed`, return \
             Poll::Ready(()); otherwise store `cx.waker().clone()` into \
             `shared.waker` and return Poll::Pending"
        )
    }
}

// =====================================================================
// STUDENT TODO (b): `join_two` combinator
//
// Poll two futures "concurrently" (interleaved within one task's
// polls — this executor is single-threaded, so there is no true
// parallelism) and resolve once BOTH have completed, yielding both
// outputs as a tuple.
//
// Pin projection note: `JoinTwo` stores each child as `Pin<Box<A>>` /
// `Pin<Box<B>>` rather than as bare `A` / `B` fields with hand-written
// (`unsafe`) pin projection. That costs one heap allocation per child.
// The payoff: `JoinTwo` itself holds nothing self-referential (its
// pinned data lives on the heap behind the `Box`es, whose addresses
// never move even if the `JoinTwo` value itself does), so it's honest
// to implement `Unpin` for it manually and poll its children through a
// plain `&mut` — no `unsafe` needed anywhere in this section. Compare
// to `futures::join!`/`pin-project`, which pin-project the fields in
// place (no extra allocation) at the cost of `unsafe` (hidden behind a
// macro). For a teaching lab, boxing trades a small, fixed allocation
// cost for a combinator you can read start to finish without auditing
// pinning invariants by hand.
//
// The struct, the `Unpin` impl, and the `join_two` constructor below
// are given. Your job is `JoinTwo::poll`:
//   - if `a_out` is still `None`, poll `a`; on `Ready`, stash the value
//     in `a_out` (and don't poll `a` again after that).
//   - do the same for `b` / `b_out`.
//   - once both `a_out` and `b_out` are `Some`, return
//     `Poll::Ready((a_out.take().unwrap(), b_out.take().unwrap()))`.
//   - otherwise return `Poll::Pending`.
// =====================================================================

/// The `Future` returned by [`join_two`].
pub struct JoinTwo<A: Future, B: Future> {
    a: Pin<Box<A>>,
    b: Pin<Box<B>>,
    a_out: Option<A::Output>,
    b_out: Option<B::Output>,
}

// SAFETY / soundness: `JoinTwo` never exposes a pinned, structural
// reference into its own fields for anything other than `a`/`b`, which
// are already independently pinned on the heap (`Pin<Box<_>>`) and
// stay valid at a fixed address regardless of whether the `JoinTwo`
// struct itself is moved. So moving a `JoinTwo` around after it has
// started being polled is always fine, and it's correct to opt back
// into `Unpin` here instead of forcing every caller to keep it pinned.
impl<A: Future, B: Future> Unpin for JoinTwo<A, B> {}

/// Poll `a` and `b` to completion, returning `(A::Output, B::Output)`
/// once both are ready.
pub fn join_two<A, B>(a: A, b: B) -> JoinTwo<A, B>
where
    A: Future,
    B: Future,
{
    JoinTwo {
        a: Box::pin(a),
        b: Box::pin(b),
        a_out: None,
        b_out: None,
    }
}

impl<A: Future, B: Future> Future for JoinTwo<A, B> {
    type Output = (A::Output, B::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!(
            "STUDENT (b): poll whichever of `a`/`b` doesn't have an \
             output yet, store outputs, and return Poll::Ready with \
             both once neither is outstanding. `JoinTwo: Unpin` (see \
             above), so `self.get_mut()` gives you a plain `&mut Self`."
        )
    }
}

// =====================================================================
// STUDENT TODO (c): the `RawWaker`/`RawWakerVTable` functions
//
// A `Waker` is, underneath, a type-erased pair: a `*const ()` data
// pointer and a `&'static RawWakerVTable` of four function pointers
// (`clone`, `wake`, `wake_by_ref`, `drop`). We make that data pointer
// really be an `Arc<Task>` (via `Arc::into_raw`), so these four
// functions are exactly the four things `Arc<Task>` needs to support.
//
// Refcount bookkeeping — read this before touching the bodies below:
//   - `Arc::into_raw(arc)` does NOT decrement the strong count; it
//     converts an owned `Arc` into a raw pointer that now "owes" that
//     same strong reference back. Every raw pointer created this way
//     must eventually flow through exactly one `Arc::from_raw` that is
//     allowed to drop (or otherwise the allocation leaks).
//   - `clone_raw`: called through `Waker::clone(&waker)` — the
//     original `Waker`/pointer stays alive afterwards, so we must NOT
//     consume its owed reference. Reconstruct the `Arc` wrapped in
//     `ManuallyDrop` (a "borrow", so it's never actually dropped here —
//     see the provided `borrow_task` helper), `Arc::clone` it to bump
//     the strong count by one, and hand the new clone's raw pointer
//     back as the new `RawWaker`'s data. Net effect: +1 strong count;
//     two pointers now each owed one reference.
//   - `wake_raw`: called through `Waker::wake(waker)`, which takes
//     `self` by value — the `Waker` is gone after this call, so this
//     function IS allowed to consume the pointer's owed reference.
//     Reconstruct a real (droppable) `Arc` (plain `Arc::from_raw`, NOT
//     `borrow_task`) and move it straight into the ready-queue send:
//     the reference this pointer owed becomes the reference the queued
//     task owns. No net change in strong count, no extra
//     clone-then-drop needed.
//   - `wake_by_ref_raw`: called through `Waker::wake_by_ref(&waker)` —
//     the `Waker` is still alive afterwards, so (like `clone_raw`) we
//     must NOT consume its owed reference. Reconstruct as a borrow
//     (`borrow_task`), and send a fresh *clone* into the queue instead
//     of moving the borrowed one.
//   - `drop_raw`: called through `Waker`'s `Drop` impl — this pointer's
//     `Waker` is going away for good, so this owes exactly one
//     `Arc::from_raw` + drop, and nothing else.
//
// Get any of these backwards and it shows up as either a double-free /
// use-after-free (dropped a reference this pointer didn't actually own)
// or a permanent leak (never dropped a reference this pointer did own)
// — both are exercised by this lab's tests indirectly, since a leaked
// or double-freed `Task` would either hang `block_on` (task never
// dropped, channel never disconnects) or crash outright.
// =====================================================================

/// Reconstruct a *borrowed* `Arc<Task>` from a raw pointer, i.e. one
/// that does NOT consume the strong reference the pointer owed.
/// Wrapping in `ManuallyDrop` means this local `Arc` is never actually
/// dropped, so letting it go out of scope pays off nothing. Provided
/// for you to use in `clone_raw` and `wake_by_ref_raw`.
///
/// # Safety
/// `ptr` must have been produced by `Arc::into_raw::<Task>` and still
/// denote a live `Task` allocation. This vtable is only ever installed
/// on `RawWaker`s built that way (see `task_waker`), so every pointer
/// the four functions below receive satisfies this.
unsafe fn borrow_task(ptr: *const ()) -> ManuallyDrop<Arc<Task>> {
    unsafe { ManuallyDrop::new(Arc::from_raw(ptr as *const Task)) }
}

unsafe fn clone_raw(ptr: *const ()) -> RawWaker {
    todo!(
        "STUDENT (c): borrow_task(ptr), Arc::clone it (bumps the \
         strong count), and return a new RawWaker wrapping \
         Arc::into_raw of the clone. Do not drop the borrow."
    )
}

unsafe fn wake_raw(ptr: *const ()) {
    todo!(
        "STUDENT (c): Arc::from_raw(ptr) to reclaim ownership, clone \
         its task_sender, and send the Arc into the queue — this \
         consumes the pointer's owed reference by transferring it to \
         the queued entry."
    )
}

unsafe fn wake_by_ref_raw(ptr: *const ()) {
    todo!(
        "STUDENT (c): borrow_task(ptr), send Arc::clone(&task) into \
         the queue (a fresh reference), and do NOT consume the \
         borrowed pointer's own reference."
    )
}

unsafe fn drop_raw(ptr: *const ()) {
    todo!(
        "STUDENT (c): Arc::from_raw(ptr) and drop it — pays off the \
         one strong reference this pointer owed."
    )
}

static TASK_VTABLE: RawWakerVTable =
    RawWakerVTable::new(clone_raw, wake_raw, wake_by_ref_raw, drop_raw);

/// Build a `Waker` for `task`. Bumps `task`'s strong count by one — the
/// reference the returned `Waker` owes, to be paid off by whichever of
/// `wake`/`drop` eventually consumes it.
fn task_waker(task: &Arc<Task>) -> Waker {
    let ptr = Arc::into_raw(Arc::clone(task)) as *const ();
    unsafe { Waker::from_raw(RawWaker::new(ptr, &TASK_VTABLE)) }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[test]
    fn test_timer_completes() {
        let result = block_on(async {
            TimerFuture::new(Duration::from_millis(20)).await;
            42
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_join_two_returns_both() {
        let (a, b) = block_on(join_two(
            async {
                TimerFuture::new(Duration::from_millis(10)).await;
                "a"
            },
            async {
                TimerFuture::new(Duration::from_millis(30)).await;
                7
            },
        ));
        assert_eq!(a, "a");
        assert_eq!(b, 7);
    }

    /// A future that records how many times it's been polled and stays
    /// `Pending` (stashing the waker it was given) until `ready` is
    /// flipped to `true`.
    struct WakeProbe {
        poll_count: Arc<AtomicUsize>,
        ready: Arc<AtomicBool>,
        waker_slot: Arc<Mutex<Option<Waker>>>,
    }

    impl Future for WakeProbe {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            self.poll_count.fetch_add(1, Ordering::SeqCst);
            if self.ready.load(Ordering::SeqCst) {
                Poll::Ready(())
            } else {
                *self.waker_slot.lock().unwrap() = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }

    #[test]
    fn test_waker_wakes_exactly_ready_task() {
        let (executor, spawner) = new_executor_and_spawner();

        let count_a = Arc::new(AtomicUsize::new(0));
        let ready_a = Arc::new(AtomicBool::new(false));
        let waker_a: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        spawner.spawn(WakeProbe {
            poll_count: Arc::clone(&count_a),
            ready: Arc::clone(&ready_a),
            waker_slot: Arc::clone(&waker_a),
        });

        let count_b = Arc::new(AtomicUsize::new(0));
        let ready_b = Arc::new(AtomicBool::new(false));
        let waker_b: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        spawner.spawn(WakeProbe {
            poll_count: Arc::clone(&count_b),
            ready: Arc::clone(&ready_b),
            waker_slot: Arc::clone(&waker_b),
        });

        // Initial tick: both tasks are freshly queued, so both get
        // polled once, both go Pending, and both stash their waker.
        executor.run_until_stalled();
        assert_eq!(count_a.load(Ordering::SeqCst), 1);
        assert_eq!(count_b.load(Ordering::SeqCst), 1);

        // Wake only A's task.
        ready_a.store(true, Ordering::SeqCst);
        waker_a.lock().unwrap().take().unwrap().wake();

        // Only A should have been resubmitted to the ready queue; B's
        // waker was never touched, so B must not be polled again.
        executor.run_until_stalled();
        assert_eq!(
            count_a.load(Ordering::SeqCst),
            2,
            "A's waker fired, so A should have been polled a second time"
        );
        assert_eq!(
            count_b.load(Ordering::SeqCst),
            1,
            "B's waker was never invoked, so B must not have been re-polled"
        );

        // Let B finish too so nothing is left dangling.
        ready_b.store(true, Ordering::SeqCst);
        waker_b.lock().unwrap().take().unwrap().wake();
        executor.run_until_stalled();
        assert_eq!(count_b.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_manual_poll_pending_then_ready() {
        // No executor at all here: build a `Context` by hand and poll
        // `TimerFuture` directly, the way `Future::poll`'s contract
        // describes, to exercise part (a) in isolation from the
        // scaffolding.
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let mut timer = Box::pin(TimerFuture::new(Duration::from_millis(30)));

        // Immediately after construction the background thread hasn't
        // had time to finish sleeping yet.
        assert_eq!(timer.as_mut().poll(&mut cx), Poll::Pending);

        // Give it comfortably longer than the timer's duration, then
        // poll again by hand — no executor, no waking, just a second
        // manual poll.
        thread::sleep(Duration::from_millis(90));
        assert_eq!(timer.as_mut().poll(&mut cx), Poll::Ready(()));
    }
}
