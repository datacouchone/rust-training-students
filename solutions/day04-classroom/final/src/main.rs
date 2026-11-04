use day04_challenge::{
    Request,
    gateway::Gateway,
    telemetry::{Event, summarize, summarize_simd},
    transport::{Fixture, request},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};
use tokio::task::JoinSet;
fn percentile(values: &mut [u64], p: usize) -> u64 {
    values.sort_unstable();
    values[(values.len() * p).div_ceil(100).saturating_sub(1)]
}
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let use_simd = std::env::var("TELEMETRY_IMPL").as_deref() == Ok("simd");
    println!("telemetry_implementation={}", if use_simd { "simd" } else { "scalar" });
    let scenario = args.get(1).map(String::as_str).unwrap_or("healthy");
    assert!(
        ["healthy", "slow", "retry", "burst", "overload", "profile"].contains(&scenario),
        "scenario: healthy | slow | retry | burst | overload | profile"
    );
    let concurrency: usize = args.get(2).map(|v| v.parse().unwrap()).unwrap_or(24);
    assert!((1..=64).contains(&concurrency));
    let seconds: u64 = args.get(3).map(|v| v.parse().unwrap()).unwrap_or(20);
    assert!((1..=120).contains(&seconds));
    let budget = Duration::from_millis(if scenario == "slow" { 50 } else { 2000 });
    let service = Gateway::new(
        4096,
        if scenario == "overload" {
            8
        } else {
            8 * 1024 * 1024
        },
        4,
        budget,
    );
    let fixture = Fixture::start(Arc::clone(&service), None).await?;
    println!(
        "pid={} gateway={} upstream={} scenario={scenario} clients={concurrency}",
        std::process::id(),
        fixture.addr,
        fixture.upstream
    );
    println!(
        "Loopback TCP; closed-loop client batch; controlled upstream application delay. Times are observations, not internet latency or an SLO."
    );
    let experiment = Instant::now();
    let mut round = 0;
    loop {
        let mut jobs = JoinSet::new();
        let wall = Instant::now();
        for i in 0..concurrency {
            let mut req = Request::sample((round * concurrency + i) as u64);
            req.fail_first = scenario == "retry";
            req.delay_ms = if scenario == "slow" { 250 } else { 10 };
            if scenario == "burst" {
                req.payload = "x".repeat(if i == 0 { 60000 } else { 16 });
            }
            let addr = fixture.addr;
            jobs.spawn(async move {
                let start = Instant::now();
                let reply = request(addr, &req).await;
                (start.elapsed().as_micros() as u64, reply)
            });
        }
        let mut latency = vec![];
        let mut queue = vec![];
        let mut route = vec![];
        let mut upstream = vec![];
        let mut outcomes = BTreeMap::new();
        let mut events = vec![];
        while let Some(result) = jobs.join_next().await {
            let (elapsed, result) = result?;
            latency.push(elapsed);
            match result {
                Ok(r) => {
                    *outcomes.entry(r.status.clone()).or_insert(0usize) += 1;
                    queue.push(r.queue_us);
                    route.push(r.route_us);
                    upstream.push(r.upstream_us);
                    events.push(Event {
                        latency: elapsed.min(u32::MAX as u64) as u32,
                        status: if r.status == "ok" { 200 } else { 503 },
                        valid: true,
                    });
                }
                Err(_) => {
                    *outcomes.entry("transport-error".into()).or_insert(0) += 1;
                }
            }
        }
        println!(
            "round={round} client_p50_us={} client_p95_us={} completed_per_s={:.1} outcomes={outcomes:?}",
            percentile(&mut latency, 50),
            percentile(&mut latency, 95),
            concurrency as f64 / wall.elapsed().as_secs_f64()
        );
        if !queue.is_empty() {
            println!(
                "stage_p95_us queue={} route={} upstream={} (percentiles are separate populations; do not add them)",
                percentile(&mut queue, 95),
                percentile(&mut route, 95),
                percentile(&mut upstream, 95)
            );
        }
        let state = service.ledger.lock().unwrap();
        println!(
            "attempts={} retries={} active_after_responses={} peak_attempts={} admitted_bytes={} accepted={} retained_capacity={} selected_errors={:?}",
            service.metrics.attempts.load(Ordering::SeqCst),
            service.metrics.retries.load(Ordering::SeqCst),
            service.metrics.active.load(Ordering::SeqCst),
            service.metrics.peak.load(Ordering::SeqCst),
            state.used,
            state.accepted,
            service.buffer.lock().unwrap().bytes.capacity(),
            if use_simd { summarize_simd(&events, 0) } else { summarize(&events, 0) }
        );
        drop(state);
        round += 1;
        if scenario != "profile" || experiment.elapsed() >= Duration::from_secs(seconds) {
            break;
        }
    }
    fixture.shutdown().await;
    // The fixture process owns its Tokio runtime. Exiting ends any leaked starter tasks.
    Ok(())
}
