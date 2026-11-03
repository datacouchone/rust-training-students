#[path = "../scenarios/tenant_limit.rs"]
mod service;
#[allow(unused_imports)]
use service::*;
#[allow(unused_imports)]
use tokio::{
    io::{AsyncWriteExt, duplex},
    time::{Duration, timeout},
};
#[tokio::main(flavor = "current_thread", start_paused = true)]
async fn main() {
    let mut g = Gateway::new(1);
    g.apply(b"1:10").unwrap();
    println!("new_tenant={:?}", g.apply(b"2:20"));
    println!("existing_tenant={:?}", g.apply(b"1:5"));
    println!("accounts={:?}", g.accounts());
}
