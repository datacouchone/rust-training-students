use case08a_profile::*;
use std::{hint::black_box, time::Instant};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("baseline");
    assert!(
        [
            "baseline",
            "compare",
            "profile-baseline",
            "profile-candidate"
        ]
        .contains(&mode)
    );
    let rows = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(20_000);
    let routes = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(512);
    assert!(rows <= 2_000_000 && routes <= 20_000 && routes > 0);
    let (policy, input) = fixture(rows, routes);
    let build = Instant::now();
    let index = if mode == "baseline" || mode == "profile-baseline" {
        None
    } else {
        Some(Index::compile(&policy).unwrap())
    };
    let setup = build.elapsed().as_nanos();
    eprintln!(
        "rows={rows} rules={} bytes={} index_build_ns={setup}",
        policy.len(),
        input.len()
    );
    if mode.starts_with("profile-") {
        let until = Instant::now() + std::time::Duration::from_secs(3);
        while Instant::now() < until {
            let req = parse_requests(black_box(&input));
            let out = if let Some(ref idx) = index {
                indexed_requests(idx, &req)
            } else {
                scan_requests(&policy, &req)
            };
            black_box(summarize(&out));
        }
        return;
    }
    let expected = scan_requests(&policy, &parse_requests(&input));
    println!("sample,implementation,parse_ns,route_ns,aggregate_ns,total_ns");
    for sample in 0..7 {
        let order = if sample % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        };
        for fast in order {
            if fast && index.is_none() {
                continue;
            }
            let total = Instant::now();
            let t = Instant::now();
            let req = parse_requests(black_box(&input));
            let parse = t.elapsed().as_nanos();
            let t = Instant::now();
            let out = if fast {
                indexed_requests(index.as_ref().unwrap(), &req)
            } else {
                scan_requests(&policy, &req)
            };
            let route = t.elapsed().as_nanos();
            let t = Instant::now();
            let summary = summarize(&out);
            black_box(&summary);
            let aggregate = t.elapsed().as_nanos();
            let elapsed = total.elapsed().as_nanos();
            assert_eq!(out, expected);
            assert_eq!(summary.values().sum::<usize>(), rows);
            println!(
                "{sample},{},{parse},{route},{aggregate},{elapsed}",
                if fast { "indexed" } else { "linear" }
            );
        }
    }
}
