use day04_challenge::telemetry::{Event,Summary,summarize,summarize_simd};
use std::{hint::black_box,time::Instant};
fn main(){
 println!("rows,distribution,threshold,sample,implementation,iterations,ns_per_scan,count,total");
 for n in [7usize,128,4099,100003] {for dense in [false,true] {
 let e:Vec<_>=(0..n).map(|i|Event{latency:((i as u64*7919)%100000)as u32,status:if dense||i%19==0 {503}else{200},valid:i%4!=0}).collect();
 for threshold in [0,50000] {let expected=summarize(&e,threshold);let iterations=(1_000_000/n.max(1)).clamp(1,10000);
 let funcs:[(&str,fn(&[Event],u32)->Summary);2]=[("scalar",summarize),("simd",summarize_simd)];
 for sample in 0..7 {for step in 0..2 {let (name,f)=funcs[(sample+step)%2];assert_eq!(f(&e,threshold),expected);
 let now=Instant::now();for _ in 0..iterations {black_box(f(black_box(&e),black_box(threshold)));}
 println!("{n},{},{threshold},{sample},{name},{iterations},{},{},{}",if dense{"many_errors"}else{"rare_errors"},now.elapsed().as_nanos()/iterations as u128,expected.count,expected.total);
 }}}}}
}
