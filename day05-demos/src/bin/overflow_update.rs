#[path = "../scenarios/overflow_update.rs"]
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
    g.apply(format!("1:{}", u64::MAX).as_bytes()).unwrap();
    let before = g.accounts().clone();
    let result = g.apply(b"1:1");
    println!(
        "result={result:?} before={before:?} after={:?}",
        g.accounts()
    );
}
