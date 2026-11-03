#![feature(portable_simd)]
use case08c_simd::*;
use std::simd::{Simd,Mask};
#[test] fn mask_keeps_exactly_lanes_4_5_6() {
 let l=Simd::from_array([10,100,200,300,400,500,600,700]);
 let s=Simd::from_array([503,200,503,600,599,500,503,503]);
 let v=Mask::from_array([true,true,false,true,true,true,true,false]);
 assert_eq!(predicate_mask(l,s,v,Query{min_latency:100,min_status:500}).to_bitmask(),0b01110000);
}
#[test] fn reduction_widens_before_sum() {
 let x=reduce_selected(Simd::splat(u32::MAX),Mask::splat(true));
 assert_eq!(x,Summary{matched:8,total_latency:8*u64::from(u32::MAX)});
 let x=reduce_selected(Simd::splat(u32::MAX),Mask::splat(false));assert_eq!(x,Summary::default());
}
#[test] fn tail_uses_global_bitmap_index() {
 let l=[100;11];let s=[503;11];let c=Columns{latency:&l,status:&s,validity:&[0,0b00000101]};
 assert_eq!(scan_simd(c,Query{min_latency:100,min_status:500}),Ok(Summary{matched:2,total_latency:200}));
}
