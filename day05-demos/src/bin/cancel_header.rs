#[path = "../scenarios/cancel_header.rs"]
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
    let bytes = frame(b"1:100");
    let (mut tx, mut rx) = duplex(64);
    let mut d = Decoder::default();
    tx.write_all(&bytes[..2]).await.unwrap();
    let result = timeout(Duration::from_millis(1), d.next(&mut rx)).await;
    println!(
        "timeout={} consumed_progress={}",
        result.is_err(),
        d.buffered()
    );
    tx.write_all(&bytes[2..]).await.unwrap();
    drop(tx);
    println!("resumed={:?}", d.next(&mut rx).await);
}
