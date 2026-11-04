use day04_challenge::telemetry::{Event,Summary,summarize,summarize_simd};
#[test] fn mask_excludes_null_low_latency_and_invalid_status() {
 let l=[10,100,200,300,400,500,600,700];let s=[503,200,503,600,599,500,503,503];let v=[true,true,false,true,true,true,true,false];
 let e:Vec<_>=(0..8).map(|i|Event{latency:l[i],status:s[i],valid:v[i]}).collect();
 assert_eq!(summarize_simd(&e,100),Summary{count:3,total:1500});
}
#[test] fn wide_sum_is_exact_across_vectors_and_tail() {
 let e=vec![Event{latency:u32::MAX,status:599,valid:true};17];
 assert_eq!(summarize_simd(&e,u32::MAX),Summary{count:17,total:17*u64::from(u32::MAX)});
}
#[test] fn every_tail_and_threshold_matches_scalar() {
 for n in 0..=73 {let e:Vec<_>=(0..n).map(|i|Event{latency:(i*7919) as u32,status:if i%3==0 {503}else{200},valid:i%4!=0}).collect();
 for threshold in [0,100,50000,u32::MAX] {assert_eq!(summarize_simd(&e,threshold),summarize(&e,threshold),"n={n}");}}
}
#[test] fn offset_event_slice_and_all_nulls() {
 let e=vec![Event{latency:u32::MAX,status:503,valid:false};20];assert_eq!(summarize_simd(&e[1..18],0),Summary{count:0,total:0});
 let e=vec![Event{latency:100,status:503,valid:true};20];assert_eq!(summarize_simd(&e[1..18],100),summarize(&e[1..18],100));
}
#[test] fn deterministic_differential_workloads() {
 let mut seed=17u64;
 for n in [7,8,9,127,128,129,4099] {let e:Vec<_>=(0..n).map(|_|{seed=seed.wrapping_mul(6364136223846793005).wrapping_add(1);Event{latency:(seed>>32)as u32,status:((seed>>16)%800)as u16,valid:seed&3!=0}}).collect();
 for threshold in [0,1_000_000,u32::MAX] {assert_eq!(summarize_simd(&e,threshold),summarize(&e,threshold));}}
}
