#[path = "../scenarios/prepare_commit.rs"]
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
    g.apply(format!("1:{}", u64::MAX - 1).as_bytes()).unwrap();
    let before = g.accounts().clone();
    let result = g.apply(b"1:2");
    println!("reject={result:?} unchanged={}", g.accounts() == &before);
    println!(
        "A checked addition is only one check. Earlier mutations also belong to the transaction."
    );
}
