#[path = "../scenarios/stream_boundaries.rs"]
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
    let (mut tx, mut rx) = duplex(64);
    tx.write_all(&[frame(b"1:10"), frame(b"2:20")].concat())
        .await
        .unwrap();
    drop(tx);
    let mut d = Decoder::default();
    println!(
        "first={:?} second={:?} end={:?}",
        d.next(&mut rx).await,
        d.next(&mut rx).await,
        d.next(&mut rx).await
    );
    println!(
        "Two complete frames work in both versions. This traffic pattern does not test cancellation."
    );
}
