#![feature(portable_simd)]
use std::simd::{Simd, Mask, Select, cmp::SimdPartialOrd, num::SimdUint};
fn main() {
    let a=[10u32,100,200,300,400,500,600,700];
    let b=[503u32,200,503,600,599,500,503,503];
    let bits=0b01111011u8;
    let latency=Simd::<u32,8>::from_array(a);let status=Simd::<u32,8>::from_array(b);
    let valid=Mask::<i32,8>::from_array(std::array::from_fn(|i|bits&(1<<i)!=0));
    let lat=latency.simd_ge(Simd::splat(100));
    let low=status.simd_ge(Simd::splat(500));let high=status.simd_le(Simd::splat(599));
    let mask=valid&lat&low&high;
    println!("lane latency status valid latency>=100 status>=500 status<=599 SELECTED");
    for i in 0..8 {println!("{i:4} {:7} {:6} {:5} {:12} {:11} {:11} {}",a[i],b[i],valid.to_array()[i],lat.to_array()[i],low.to_array()[i],high.to_array()[i],mask.to_array()[i]);}
    let selected=mask.select(latency,Simd::splat(0)).cast::<u64>();
    println!("selected lanes: {:?}",selected.to_array());
    println!("bitmask={:08b} (printed rightmost bit is lane 0); count={}; sum={}",mask.to_bitmask(),mask.to_bitmask().count_ones(),selected.reduce_sum());
    assert_eq!(mask.to_bitmask(),0b01110000);assert_eq!(selected.reduce_sum(),1500);
    println!("11 rows: vector indices 0..8; tail indices 8,9,10; tail bitmap uses byte i/8 and bit i%8");
    println!("8 * u32::MAX = {}: widen each lane BEFORE reducing",8*u64::from(u32::MAX));
}
