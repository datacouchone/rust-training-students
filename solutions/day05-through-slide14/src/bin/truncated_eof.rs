#[path = "../scenarios/truncated_eof.rs"]
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
    for input in [vec![], frame(b""), vec![0, 0]] {
        let (mut tx, mut rx) = duplex(64);
        tx.write_all(&input).await.unwrap();
        drop(tx);
        println!(
            "wire={input:?} result={:?}",
            Decoder::default().next(&mut rx).await
        );
    }
}
