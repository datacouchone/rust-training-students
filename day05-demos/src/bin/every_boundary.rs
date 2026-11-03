#[path = "../scenarios/every_boundary.rs"]
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
    let mut preserved = 0;
    for split in 1..bytes.len() {
        let (mut tx, mut rx) = duplex(64);
        let mut d = Decoder::default();
        tx.write_all(&bytes[..split]).await.unwrap();
        let _ = timeout(Duration::from_millis(1), d.next(&mut rx)).await;
        tx.write_all(&bytes[split..]).await.unwrap();
        drop(tx);
        let ok = matches!(d.next(&mut rx).await,Ok(Some(ref b)) if b==b"1:100");
        preserved += usize::from(ok);
        println!("split={split} preserved={ok}");
    }
    println!("preserved={preserved}/{}", bytes.len() - 1);
}
