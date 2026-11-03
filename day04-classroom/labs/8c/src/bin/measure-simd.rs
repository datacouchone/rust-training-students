use case08c_simd::*;
use std::{hint::black_box, time::Instant};
fn main() {
    let compare = std::env::args().nth(1).as_deref() == Some("compare");
    let large = std::env::args()
        .nth(2)
        .map(|v| v.parse().unwrap())
        .unwrap_or(1_000_003);
    assert!(large <= 10_000_000);
    println!("length,sparse,sample,implementation,iterations,elapsed_ns,matched,total_latency");
    for n in [7, 128, 4099, large] {
        for sparse in [false, true] {
            let c = fixture(n, sparse);
            let q = Query {
                min_latency: 50_000,
                min_status: 500,
            };
            let expected = scan_scalar(c.view(), q).unwrap();
            let funcs: [(&str, fn(Columns<'_>, Query) -> Result<Summary, Error>); 4] = [
                ("scalar", scan_scalar),
                ("unrolled", scan_unrolled),
                ("simd", scan_simd),
                ("auto", scan_auto),
            ];
            let iterations = (1_000_000usize / n.max(1)).clamp(1, 10_000);
            for sample in 0..7 {
                for step in 0..if compare { 4 } else { 1 } {
                    let index = if compare { (sample + step) % 4 } else { 0 };
                    let (name, f) = funcs[index];
                    assert_eq!(f(c.view(), q).unwrap(), expected);
                    let start = Instant::now();
                    for _ in 0..iterations {
                        black_box(f(black_box(c.view()), black_box(q)).unwrap());
                    }
                    let elapsed = start.elapsed().as_nanos();
                    println!(
                        "{n},{sparse},{sample},{name},{iterations},{elapsed},{},{}",
                        expected.matched, expected.total_latency
                    );
                }
            }
        }
    }
}
