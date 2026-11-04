//! A real async function, with logging around each call to its Future::poll.
use std::future::{Future, poll_fn};
use tokio::time::{Duration, sleep};

async fn upstream_request() -> &'static str {
    println!("  A: start; waiting for the first timer (1 second)");

    // The timer can return Pending. Our async function then also returns Pending
    // to its caller, preserving where it stopped and any state it still owns.
    sleep(Duration::from_secs(1)).await;

    // Resume here after the first wake, then create the second timer.
    println!("  B: first wait finished; waiting for the second timer (2 seconds)");
    sleep(Duration::from_secs(2)).await;

    // Another poll resumes here. Neither A nor B is executed again.
    println!("  C: second wait finished");
    "response"
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Calling the async function creates its future; the body has not run.
    // Box::pin gives us the pinned access required by Future::poll.
    let mut request = Box::pin(upstream_request());
    println!("main: future created; request body has not run yet");
    let mut polls = 0;

    // Normally we would write `request.await`. This wrapper exposes the polls.
    // poll_fn makes a future whose poll calls this closure; it is not a scheduler.
    let response = poll_fn(|cx| {
        polls += 1;
        println!("[poll {polls}] polling the request");
        // Pass the current task's context/waker down to the async future.
        let state = request.as_mut().poll(cx);
        println!("[poll {polls}] returned {state:?}");
        // Pending propagates to the caller. This is NOT a busy-polling loop.
        state
    })
    .await;

    println!("main: received {response:?} after {polls} polls");
}

// The timer arranges a wake when its deadline is reached. Tokio then drives the
// outer future again, which reaches this wrapper and polls the request again.
// The exact number of polls is not a contract; the request body resumes from
// its saved state rather than starting from the first println every time.
