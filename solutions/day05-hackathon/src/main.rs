use day05_hackathon::{
    analytics::{Buffer, Row, summarize},
    boundary::{self, NativeSummary},
    codec::{Decoder, frame},
    fixture::{Server, send},
    rollout::{Window, decide},
    worker::Service,
};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tokio::{
    io::{AsyncWriteExt, duplex},
    task::JoinSet,
    time::{Instant, timeout},
};
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("load");
    assert!(
        [
            "load", "slow", "replay", "burst", "query", "rollout", "fragment", "layout"
        ]
        .contains(&mode),
        "unknown scenario"
    );
    let n: usize = args
        .get(2)
        .map(|s| s.parse().expect("integer client count"))
        .unwrap_or(24);
    assert!((1..=48).contains(&n));
    timeout(Duration::from_secs(15), run(mode, n))
        .await
        .expect("scenario watchdog");
}
async fn run(mode: &str, n: usize) {
    println!(
        "scenario={mode} pid={} clients={n} runtime_workers=2",
        std::process::id()
    );
    match mode {
        "layout" => {
            println!(
                "C ABI: size={} align={} count_offset={}; explicit wire prefix=21 bytes",
                std::mem::size_of::<NativeSummary>(),
                std::mem::align_of::<NativeSummary>(),
                std::mem::offset_of!(NativeSummary, count)
            );
            println!("Do not serialize struct padding. Native layout is platform-specific.");
            return;
        }
        "fragment" => {
            let bytes = frame(b"1:10");
            let (mut tx, mut rx) = duplex(64);
            let mut d = Decoder::default();
            tx.write_all(&bytes[..2]).await.unwrap();
            let first = timeout(Duration::from_millis(10), d.next(&mut rx)).await;
            println!(
                "first_wait_timed_out={} retained_bytes={}",
                first.is_err(),
                d.buffered()
            );
            tx.write_all(&bytes[2..]).await.unwrap();
            drop(tx);
            println!("resumed={:?}", d.next(&mut rx).await);
            return;
        }
        "burst" => {
            let mut b = Buffer {
                bytes: vec![1; 65_536],
                retain: 4096,
            };
            println!(
                "before len={} capacity={}",
                b.bytes.len(),
                b.bytes.capacity()
            );
            b.flush();
            println!(
                "after len={} capacity={} retain={}",
                b.bytes.len(),
                b.bytes.capacity(),
                b.retain
            );
            return;
        }
        "query" => {
            let rows = [
                Row {
                    valid: true,
                    status: 503,
                    latency: 800,
                },
                Row {
                    valid: false,
                    status: 503,
                    latency: 900,
                },
                Row {
                    valid: true,
                    status: 600,
                    latency: 1000,
                },
            ];
            println!("query={:?}", summarize(&rows, 500));
            return;
        }
        "rollout" => {
            for (name, w) in [
                (
                    "tiny",
                    Window {
                        requests: 10,
                        successes: 10,
                        errors: 0,
                        p95_us: 1000,
                        duplicates: 0,
                    },
                ),
                (
                    "duplicate",
                    Window {
                        requests: 1000,
                        successes: 1000,
                        errors: 0,
                        p95_us: 1000,
                        duplicates: 1,
                    },
                ),
                (
                    "healthy",
                    Window {
                        requests: 1000,
                        successes: 995,
                        errors: 5,
                        p95_us: 20_000,
                        duplicates: 0,
                    },
                ),
            ] {
                println!("window={name} decision={:?}", decide(w));
            }
            return;
        }
        _ => {}
    }
    let service = Service::new(4, 8, 128);
    let slow = mode == "slow";
    let delay = Duration::from_millis(if slow { 250 } else { 10 });
    let budget = Duration::from_millis(if slow { 30 } else { 2000 });
    let server = Server::start(Arc::clone(&service), delay, budget)
        .await
        .unwrap();
    println!(
        "gateway={} local TCP; delay is enrichment wait, not network RTT",
        server.address
    );
    if mode == "replay" {
        let event = boundary::encode(7, 42, 100, "/orders");
        let first = send(server.address, &event).await.unwrap();
        println!(
            "first_response={first}; client treats ACK as lost and resends identical identity"
        );
        println!(
            "second_response={}",
            send(server.address, &event).await.unwrap()
        );
        println!("account={:?}", service.ledger.lock().await.accounts.get(&7));
        return;
    }
    let start = Instant::now();
    let mut tasks = JoinSet::new();
    for id in 0..n {
        let address = server.address;
        tasks.spawn(async move {
            let t = Instant::now();
            let code = send(address, &boundary::encode(7, id as u64, 100, "/orders")).await;
            (t.elapsed().as_micros(), code)
        });
    }
    let mut times = vec![];
    let mut outcomes = [0usize; 5];
    while let Some(result) = tasks.join_next().await {
        let (elapsed, code) = result.unwrap();
        times.push(elapsed);
        outcomes[code.unwrap_or(4) as usize] += 1;
    }
    let elapsed = start.elapsed();
    times.sort_unstable();
    println!(
        "outcomes applied/replay/rejected/timeout/transport_error={outcomes:?} p50_us={} p95_us={} completed_per_s={:.1}",
        times[(n - 1) / 2],
        times[((n * 95).div_ceil(100) - 1).min(n - 1)],
        n as f64 / elapsed.as_secs_f64()
    );
    println!(
        "at_responses active={} peak={} available_slots={}",
        service.active.load(Ordering::SeqCst),
        service.peak.load(Ordering::SeqCst),
        service.slots.available_permits()
    );
    tokio::time::sleep(Duration::from_millis(if slow {
        n as u64 * 250 + 50
    } else {
        20
    }))
    .await;
    println!(
        "after_settle active={} ledger={:?}",
        service.active.load(Ordering::SeqCst),
        service.ledger.lock().await.accounts
    );
    println!(
        "Small closed-loop batch; outcomes matter. One observation is not a production SLO or CPU profile."
    );
}
