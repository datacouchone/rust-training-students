use case08c_simd::*;
use std::{hint::black_box,time::Instant};
fn main() {
 println!("stage,n,sparse,sample,implementation,iterations,ns_per_scan,matched,total");
 // Identical bounded inputs at EVERY stage: valid shape, multiples of eight, no u32 lane-sum overflow.
 for n in [128,4096] {for sparse in [false,true] {
 let c=fixture(n,sparse);let q=Query{min_latency:50000,min_status:500};let expected=scan_scalar(c.view(),q).unwrap();
 let runs=500;
 for sample in 0..7 {for step in 0..2 {let candidate=(sample+step)%2==1;
 let f=if candidate {scan_simd} else {scan_scalar};assert_eq!(f(c.view(),q).unwrap(),expected);
 let now=Instant::now();for _ in 0..runs {black_box(f(black_box(c.view()),black_box(q)).unwrap());}
 println!("{STAGE},{n},{sparse},{sample},{},{runs},{},{},{}",if candidate {"candidate"}else{"scalar"},now.elapsed().as_nanos()/runs,expected.matched,expected.total_latency);
 }} }}
}
