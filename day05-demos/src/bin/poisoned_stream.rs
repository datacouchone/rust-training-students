#[path = "../scenarios/poisoned_stream.rs"]
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
    tx.write_all(&u32::MAX.to_be_bytes()).await.unwrap();
    let mut d = Decoder::default();
    println!("oversize={:?}", d.next(&mut rx).await);
    println!(
        "retry={:?}",
        timeout(Duration::from_millis(1), d.next(&mut rx)).await
    );
    println!("An invalid stream must fail immediately on reuse, not wait for a new header.");
}
