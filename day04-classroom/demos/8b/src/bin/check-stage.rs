use case08b_memory::*;
fn main() {
    println!("MEMORY STAGE {STAGE}: checks apply only to concepts installed so far");
    let mut b=Batch::new(Limits::default());
    let a=b.push(7,"/same",10).unwrap(); let z=b.push(8,"/same",20).unwrap();
    assert_eq!(b.get(a).unwrap().tenant,7); assert_eq!(b.get(z).unwrap().tenant,8);
    assert_eq!(b.footprint().rows,2);
    println!("two distinct rows, stored_path_bytes={} (stage 0 duplicates bytes)",b.footprint().payload_bytes);
    if STAGE>=1 {assert_eq!(b.footprint().payload_bytes,5);}
    let mut collision=Batch::with_hasher(Limits::default(), |_| 0);
    collision.push(1,"/alpha",1).unwrap();let t=collision.push(2,"/beta",2).unwrap();
    let collision_ok=collision.get(t).unwrap().path=="/beta";
    println!("forced collision preserves /beta: {collision_ok}");
    if STAGE>=2 {assert!(collision_ok);} else if STAGE==1 {assert!(!collision_ok);println!("EXPECTED LIMITATION: stage 1 aliases colliding paths; do not deploy");}
    let mut quota=Batch::new(Limits {rows:1,..Limits::default()});
    quota.push(1,"/a",1).unwrap();let before=quota.footprint();
    let rejected=quota.push(2,"/b",1).is_err();
    println!("row quota rejected={rejected}");
    if STAGE>=3 {assert!(rejected);assert_eq!(quota.footprint(),before);}
    if STAGE>=3 {
        let mut overflow=Batch::new(Limits::default());overflow.push(1,"/a",u64::MAX).unwrap();
        let before=overflow.footprint();assert_eq!(overflow.push(2,"/new",1),Err(Error::Overflow));
        assert_eq!(overflow.footprint(),before);println!("overflow rejected without interning /new");
    }
    b.reset().unwrap();b.push(9,"/new",1).unwrap();
    let stale_rejected=b.get(a).is_none();println!("old ticket rejected={stale_rejected}");
    if STAGE>=4 {assert!(stale_rejected);}
    let mut burst=Batch::new(Limits{rows:10000,bytes:1_000_000,paths:10000,retain_rows:4,retain_bytes:128,retain_buckets:4});
    for (t,p,n) in fixture(1000,1000) {burst.push(t,&p,n).unwrap();}
    burst.reset().unwrap();let retained=burst.footprint();println!("after burst reset: {retained:?}");
    if STAGE>=5 {assert_eq!(retained.payload_capacity,0);assert_eq!(retained.row_capacity,0);assert_eq!(retained.bucket_capacity,0);}
    println!("PASS: stage-specific assertions; earlier stages are not the complete lab contract");
}
