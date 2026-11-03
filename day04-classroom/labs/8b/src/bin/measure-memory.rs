use case08b_memory::*;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
struct Meter;
static CALLS: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
fn added(n: usize) {
    CALLS.fetch_add(1, Ordering::Relaxed);
    let live = LIVE.fetch_add(n, Ordering::Relaxed) + n;
    PEAK.fetch_max(live, Ordering::Relaxed);
}
// SAFETY: delegates unchanged allocation contracts to System; atomic accounting never allocates.
unsafe impl GlobalAlloc for Meter {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            added(l.size());
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Ordering::Relaxed);
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let q = unsafe { System.realloc(p, l, n) };
        if !q.is_null() {
            LIVE.fetch_sub(l.size(), Ordering::Relaxed);
            added(n);
        }
        q
    }
}
#[global_allocator]
static ALLOC: Meter = Meter;
fn main() {
    let compare = std::env::args().nth(1).as_deref() == Some("compare");
    let n = std::env::args()
        .nth(2)
        .map(|s| s.parse().unwrap())
        .unwrap_or(10_000);
    assert!(n <= 100_000 && n > 0);
    println!(
        "workload,sample,implementation,elapsed_ns,allocation_calls,peak_requested_bytes_delta,retained_requested_bytes_delta,retained_payload_capacity,total"
    );
    for (name, unique) in [("repeated", 8), ("distinct", n), ("burst_then_idle", n)] {
        let input = fixture(n, unique);
        let expected: u64 = input.iter().map(|r| r.2).sum();
        for sample in 0..7 {
            for fast in if sample % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                if fast && !compare {
                    continue;
                }
                let base = LIVE.load(Ordering::Relaxed);
                CALLS.store(0, Ordering::Relaxed);
                PEAK.store(base, Ordering::Relaxed);
                let start = Instant::now();
                if fast {
                    let limits = Limits {
                        rows: n,
                        bytes: n * 64,
                        paths: n,
                        retain_bytes: if name == "burst_then_idle" {
                            128
                        } else {
                            n * 64
                        },
                        retain_rows: if name == "burst_then_idle" { 4 } else { n * 2 },
                        retain_buckets: if name == "burst_then_idle" { 4 } else { n * 2 },
                    };
                    let mut batch = Batch::new(limits);
                    let mut first = None;
                    for (tenant, path, bytes) in &input {
                        let t = batch.push(*tenant, path, *bytes).unwrap();
                        first.get_or_insert(t);
                    }
                    let total = batch.total_bytes();
                    black_box(batch.get(first.unwrap()));
                    batch.reset().unwrap();
                    let elapsed = start.elapsed().as_nanos();
                    let calls = CALLS.load(Ordering::Relaxed);
                    let peak = PEAK.load(Ordering::Relaxed).saturating_sub(base);
                    let retained = LIVE.load(Ordering::Relaxed).saturating_sub(base);
                    let capacity = batch.footprint().payload_capacity;
                    assert_eq!(total, expected);
                    println!(
                        "{name},{sample},batch,{elapsed},{calls},{peak},{retained},{capacity},{total}"
                    );
                } else {
                    let (mut owned, total) = owned_baseline(black_box(&input));
                    black_box(&owned);
                    owned.clear();
                    let elapsed = start.elapsed().as_nanos();
                    let calls = CALLS.load(Ordering::Relaxed);
                    let peak = PEAK.load(Ordering::Relaxed).saturating_sub(base);
                    let retained = LIVE.load(Ordering::Relaxed).saturating_sub(base);
                    assert_eq!(total, expected);
                    println!("{name},{sample},owned,{elapsed},{calls},{peak},{retained},0,{total}");
                    black_box(owned);
                }
            }
        }
    }
}
