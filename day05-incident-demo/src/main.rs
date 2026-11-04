//! Instructor-led, bounded demonstrations over real localhost TCP.
//! The existing lab stays untouched. BEFORE/AFTER select named implementations.
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Barrier, Mutex},
    task::JoinSet,
    time::{sleep, timeout},
};
use tracing::Instrument;
const LIMIT: Duration = Duration::from_millis(250);

// A guard counts the real lifetime of each owned background future.
struct Live(Arc<AtomicUsize>);
impl Live {
    fn new(n: Arc<AtomicUsize>) -> Self {
        n.fetch_add(1, Ordering::SeqCst);
        Self(n)
    }
}
impl Drop for Live {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
#[derive(Clone)]
struct State {
    metrics: Arc<Mutex<usize>>,
    registry: Arc<Mutex<usize>>,
    active: Arc<AtomicUsize>,
    tasks: Arc<Mutex<JoinSet<()>>>,
}
impl State {
    fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(0)),
            registry: Arc::new(Mutex::new(0)),
            active: Arc::new(AtomicUsize::new(0)),
            tasks: Arc::new(Mutex::new(JoinSet::new())),
        }
    }
    async fn cleanup(&self) {
        self.tasks.lock().await.shutdown().await;
    }
}

// BEFORE: force both tasks to own their first lock, then request the other.
// The barrier makes the cycle deterministic instead of relying on sleeps.
async fn deadlock_before(s: &State) {
    let rendezvous = Arc::new(Barrier::new(3));
    let mut tasks = s.tasks.lock().await;
    for reverse in [false, true] {
        let s = s.clone();
        let b = rendezvous.clone();
        tasks.spawn(async move {
            let _live = Live::new(s.active.clone());
            let (first, second) = if reverse {
                (&s.registry, &s.metrics)
            } else {
                (&s.metrics, &s.registry)
            };
            let _first = first.lock().await;
            tracing::info!(
                task = if reverse { "B" } else { "A" },
                held = if reverse { "registry" } else { "metrics" },
                "first lock acquired"
            );
            b.wait().await;
            tracing::info!(
                task = if reverse { "B" } else { "A" },
                waiting = if reverse { "metrics" } else { "registry" },
                "requesting second lock"
            );
            let _second = second.lock().await;
        });
    }
    drop(tasks);
    rendezvous.wait().await;
}
// AFTER: both tasks use metrics -> registry. The waiting task holds neither.
async fn deadlock_after(s: &State) {
    let mut tasks = JoinSet::new();
    for name in ["A", "B"] {
        let s = s.clone();
        tasks.spawn(async move {
            let _metrics = s.metrics.lock().await;
            let _registry = s.registry.lock().await;
            tracing::info!(
                task = name,
                "both locks acquired in metrics -> registry order"
            );
        });
    }
    while let Some(result) = tasks.join_next().await {
        result.unwrap();
    }
}
// BEFORE: a completed request leaves a never-ending child future alive.
// The DEMO supervisor retains its handle only so this demonstration can clean up.
// The original server.rs:128 discards the handle; that is why it cannot join it.
async fn leak_before(s: &State) {
    let (ready, started) = tokio::sync::oneshot::channel();
    let active = s.active.clone();
    s.tasks.lock().await.spawn(async move {
        let _live = Live::new(active);
        let _ = ready.send(());
        loop {
            sleep(Duration::from_secs(3600)).await;
        }
    });
    started.await.unwrap();
}
// AFTER: the child has a finite job; the request owns and joins it before replying.
async fn leak_after(s: &State) {
    let active = s.active.clone();
    tokio::spawn(async move {
        let _live = Live::new(active);
        sleep(Duration::from_millis(5)).await;
    })
    .await
    .unwrap();
}
// BEFORE: an expected parse error becomes a panic in a connection task.
fn panic_before(body: &str) -> String {
    let n = body.parse::<u64>().unwrap();
    format!("OK {n}")
}
// AFTER: malformed external input gets an error response; the task survives.
fn panic_after(body: &str) -> String {
    match body.parse::<u64>() {
        Ok(n) => format!("OK {n}"),
        Err(e) => {
            tracing::warn!(error=%e,"rejecting malformed input");
            "ERR invalid number".into()
        }
    }
}

async fn read_frame(s: &mut TcpStream) -> std::io::Result<String> {
    let n = s.read_u32().await? as usize;
    if n > 4096 {
        return Err(std::io::Error::other("frame limit"));
    }
    let mut data = vec![0; n];
    s.read_exact(&mut data).await?;
    String::from_utf8(data).map_err(std::io::Error::other)
}
async fn write_frame(s: &mut TcpStream, body: &str) -> std::io::Result<()> {
    s.write_u32(body.len() as u32).await?;
    s.write_all(body.as_bytes()).await
}
#[tracing::instrument(skip(socket, state), fields(frame_no = 1))]
async fn serve_one(
    mut socket: TcpStream,
    state: State,
    fault: String,
    repaired: bool,
    conn_id: usize,
) {
    let body = match read_frame(&mut socket).await {
        Ok(b) => b,
        Err(_) => return,
    };
    let response = match fault.as_str() {
        "deadlock" => {
            let _metrics = state.metrics.lock().await;
            "OK".into()
        }
        "leak" => {
            if repaired {
                leak_after(&state).await
            } else {
                leak_before(&state).await
            };
            "OK".into()
        }
        "panic" => {
            if repaired {
                panic_after(&body)
            } else {
                panic_before(&body)
            }
        }
        _ => unreachable!(),
    };
    tracing::info!(%response,"sending response");
    let _ = write_frame(&mut socket, &response).await;
}
#[derive(Debug)]
struct Observation {
    replies: Vec<String>,
    timeouts: usize,
    panics: usize,
    live: usize,
}
async fn scenario(fault: &str, repaired: bool) -> Observation {
    println!(
        "\n=== {} {} · real TCP ===",
        fault,
        if repaired { "AFTER" } else { "BEFORE" }
    );
    let state = State::new();
    if fault == "deadlock" {
        if repaired {
            deadlock_after(&state).await
        } else {
            deadlock_before(&state).await
        }
    }
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let requests = if fault == "leak" {
        3
    } else if fault == "panic" {
        2
    } else {
        1
    };
    let mut observation = Observation {
        replies: vec![],
        timeouts: 0,
        panics: 0,
        live: 0,
    };
    for id in 1..=requests {
        let mut client = TcpStream::connect(address).await.unwrap();
        let (socket, _) = listener.accept().await.unwrap();
        let mut task = tokio::spawn(
            serve_one(socket, state.clone(), fault.into(), repaired, id)
                .instrument(tracing::info_span!("connection", conn_id = id)),
        );
        let body = if fault == "panic" && id == 1 {
            "not-a-number"
        } else {
            "42"
        };
        write_frame(&mut client, body).await.unwrap();
        match timeout(LIMIT, read_frame(&mut client)).await {
            Err(_) => {
                observation.timeouts += 1;
                println!("client conn_id={id} result=TIMEOUT");
                task.abort();
            }
            Ok(Err(_)) => println!("client conn_id={id} result=EOF"),
            Ok(Ok(reply)) => {
                println!("client conn_id={id} result={reply}");
                observation.replies.push(reply);
            }
        }
        match timeout(Duration::from_secs(1), &mut task).await {
            Ok(Err(e)) if e.is_panic() => {
                observation.panics += 1;
                println!("supervisor conn_id={id} join_error=PANIC process_alive=true");
            }
            Err(_) => {
                task.abort();
                let _ = task.await;
            }
            _ => {}
        }
        println!(
            "after_response live_background_tasks={}",
            state.active.load(Ordering::SeqCst)
        );
    }
    observation.live = state.active.load(Ordering::SeqCst);
    println!("observed {observation:?}");
    state.cleanup().await;
    println!(
        "demo_cleanup live_background_tasks={}",
        state.active.load(Ordering::SeqCst)
    );
    assert_eq!(state.active.load(Ordering::SeqCst), 0);
    observation
}
async fn compare(fault: &str) {
    let before = scenario(fault, false).await;
    let after = scenario(fault, true).await;
    match fault {
        "deadlock" => {
            assert_eq!(before.timeouts, 1);
            assert_eq!(before.live, 2);
            assert_eq!(after.replies, ["OK"]);
        }
        "leak" => {
            assert_eq!(before.live, 3);
            assert_eq!(after.live, 0);
            assert_eq!(before.replies, after.replies);
        }
        "panic" => {
            assert_eq!(before.panics, 1);
            assert_eq!(after.panics, 0);
            assert_eq!(after.replies, ["ERR invalid number", "OK 42"]);
        }
        _ => unreachable!(),
    }
    println!("CHECK PASS: {fault} before/after expectations; all owned demo tasks cleaned up");
}

// Real TCP retry: commit succeeds, the client discards its ACK, then retries.
async fn retry(repaired: bool) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let mut receipts = BTreeMap::<String, String>::new();
    let mut charges = 0;
    println!(
        "\n=== lost ACK {} · real TCP ===",
        if repaired { "AFTER" } else { "BEFORE" }
    );
    for attempt in 0..3 {
        let mut client = TcpStream::connect(address).await.unwrap();
        let (mut server, _) = listener.accept().await.unwrap();
        let payload = if attempt == 2 {
            "1:req-7:99"
        } else {
            "1:req-7:10"
        };
        write_frame(&mut client, payload).await.unwrap();
        let received = read_frame(&mut server).await.unwrap();
        let (key, value) = received.rsplit_once(':').unwrap();
        let response = if repaired {
            match receipts.get(key) {
                Some(old) if old == value => "REPLAY",
                Some(_) => "CONFLICT",
                None => {
                    receipts.insert(key.into(), value.into());
                    charges += 1;
                    "APPLIED"
                }
            }
        } else {
            charges += 1;
            "APPLIED"
        };
        write_frame(&mut server, response).await.unwrap();
        if attempt == 0 {
            drop(client);
            println!("attempt=0 committed=true client_discarded_ack=true charges={charges}");
        } else {
            println!(
                "attempt={attempt} response={} charges={charges}",
                read_frame(&mut client).await.unwrap()
            );
        }
    }
    assert_eq!(charges, if repaired { 1 } else { 3 });
}
#[derive(Debug)]
struct Window {
    total: u64,
    errors: u64,
    duplicate_effects: u64,
    p95_ms: u64,
}
fn gate(w: &Window) -> &'static str {
    if w.total == 0 || w.errors > w.total || w.duplicate_effects > w.total {
        "HOLD"
    } else if w.duplicate_effects > 0 {
        "ROLLBACK"
    } else if w.total < 1000 {
        "HOLD"
    } else if w.errors * 100 > w.total || w.p95_ms > 50 {
        "ROLLBACK"
    } else {
        "PROMOTE"
    }
}
fn rollout() {
    println!("Teaching windows, not measured production traffic; thresholds are exercise policy.");
    for w in [
        Window {
            total: 0,
            errors: 0,
            duplicate_effects: 0,
            p95_ms: 0,
        },
        Window {
            total: 20,
            errors: 0,
            duplicate_effects: 0,
            p95_ms: 5,
        },
        Window {
            total: 1000,
            errors: 0,
            duplicate_effects: 1,
            p95_ms: 5,
        },
        Window {
            total: 1000,
            errors: 0,
            duplicate_effects: 0,
            p95_ms: 80,
        },
        Window {
            total: 1000,
            errors: 2,
            duplicate_effects: 0,
            p95_ms: 20,
        },
    ] {
        println!("{w:?} decision={}", gate(&w));
    }
}
async fn wait_for_sample() {
    let state = State::new();
    deadlock_before(&state).await;
    println!(
        "SAMPLE_READY pid={} live_background_tasks=2 cycle=metrics->registry->metrics",
        std::process::id()
    );
    // Give runnable logging work a chance to settle; lock ownership is already
    // established by the barrier, not by this delay. Linux GDB stops at the marker.
    sleep(Duration::from_millis(100)).await;
    classroom_capture_ready();
    // Sampling harness terminates only this dedicated child. Standalone runs also expire.
    sleep(Duration::from_secs(12)).await;
    state.cleanup().await;
}

// Stable debugger breakpoint, called only by the dedicated capture child.
// SAFETY: this crate owns this unique exported symbol; the C ABI takes no arguments.
#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn classroom_capture_ready() {
    std::hint::black_box(());
}
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();
    // Keep panic visible and compact. Tokio still supplies JoinError to the supervisor.
    std::panic::set_hook(Box::new(|p| eprintln!("EXPECTED INJECTED PANIC: {p}")));
    let arg = std::env::args().nth(1).unwrap_or_else(|| "all".into());
    match arg.as_str() {
        "deadlock" | "leak" | "panic" => compare(&arg).await,
        "all" => {
            for fault in ["deadlock", "leak", "panic"] {
                compare(fault).await
            }
        }
        "retry" => {
            retry(false).await;
            retry(true).await;
        }
        "rollout" => rollout(),
        "sample-child" => wait_for_sample().await,
        _ => panic!("use all, deadlock, leak, panic, retry, rollout or sample-child"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn deadlock_trigger_and_repair() {
        compare("deadlock").await;
    }
    #[tokio::test]
    async fn request_owned_tasks_finish() {
        compare("leak").await;
    }
    #[tokio::test]
    async fn malformed_input_does_not_kill_repaired_task() {
        compare("panic").await;
    }
    #[tokio::test]
    async fn lost_ack_retry_preserves_single_effect() {
        retry(true).await;
    }
    #[test]
    fn gate_rejects_small_and_duplicate_windows() {
        assert_eq!(
            gate(&Window {
                total: 10,
                errors: 0,
                duplicate_effects: 0,
                p95_ms: 1
            }),
            "HOLD"
        );
        assert_eq!(
            gate(&Window {
                total: 1000,
                errors: 0,
                duplicate_effects: 1,
                p95_ms: 1
            }),
            "ROLLBACK"
        );
    }
}
