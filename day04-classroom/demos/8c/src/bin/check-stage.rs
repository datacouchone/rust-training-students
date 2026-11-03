use case08c_simd::*;
fn main() {
 println!("SCAN STAGE {STAGE}: incomplete stages explicitly report their limitations");
 let q=Query{min_latency:50,min_status:500};
 let c=OwnedColumns{latency:vec![100;8],status:vec![0,99,100,499,500,599,600,u32::MAX],validity:vec![255]};
 assert_eq!(scan_simd(c.view(),q),scan_scalar(c.view(),q));
 let null=OwnedColumns{latency:vec![100;8],status:vec![503;8],validity:vec![0]};
 assert_eq!(scan_simd(null.view(),q).unwrap().matched,0);
 println!("PASS complete validity/status/latency predicate on 8 rows");
 if STAGE>=1 {let bad=Columns{latency:&[1],status:&[],validity:&[1]};assert_eq!(scan_simd(bad,q),Err(Error::Length));println!("PASS malformed shape rejected");}
 if STAGE==0 {println!("LIMITATION: stage 0 assumes prevalidated input");}
 if STAGE==3 {println!("LIMITATION: sum still u32 and tails omitted; timings only use bounded 8-row multiples");}
 if STAGE==4 {println!("LIMITATION: tails still omitted");}
 if STAGE>=4 {let high=OwnedColumns{latency:vec![u32::MAX;8],status:vec![503;8],validity:vec![255]};assert_eq!(scan_simd(high.view(),q).unwrap().total_latency,8*u32::MAX as u64);println!("PASS widening before reduction");}
 if STAGE>=5 {for n in 0..=73 {for sparse in [false,true] {let c=fixture(n,sparse);assert_eq!(scan_simd(c.view(),q),scan_scalar(c.view(),q));}}println!("PASS every tail, empty input and sparse/dense validity");}
 println!("PASS current-stage checks; run full acceptance only after stage 5");
}
