//! Deterministic incident reproducer. These are locally generated observations, not company logs.
use case09_incident::*;
use tokio::io::{AsyncWriteExt, duplex};
#[tokio::main(flavor = "current_thread")]
async fn main() {
    tokio::time::pause();
    let scenario = std::env::args().nth(1).unwrap_or_else(|| "1".into());
    match scenario.as_str() {
        "1" => {
            let bytes = frame(b"12:100");
            let (mut tx, mut rx) = duplex(64);
            let mut d = Decoder::default();
            tx.write_all(&bytes[..6]).await.unwrap();
            let first =
                tokio::time::timeout(std::time::Duration::from_millis(10), d.next(&mut rx)).await;
            println!(
                "event=request_deadline elapsed={} buffered={}",
                first.is_err(),
                d.buffered()
            );
            tx.write_all(&bytes[6..]).await.unwrap();
            drop(tx);
            println!("event=retry result={:?}", d.next(&mut rx).await);
        }
        "2" => {
            let mut g = Gateway::new(2);
            g.apply(format!("1:{}", u64::MAX).as_bytes()).unwrap();
            println!("event=before accounts={:?}", g.accounts());
            println!("event=request result={:?}", g.apply(b"1:1"));
            println!("event=after accounts={:?}", g.accounts());
        }
        "3" => {
            let mut g = Gateway::new(2);
            for id in 0..5 {
                println!(
                    "event=request id={id} result={:?}",
                    g.apply(format!("{id}:10").as_bytes())
                );
            }
            println!("event=footprint tenants={}", g.accounts().len());
        }
        _ => {
            eprintln!("choose scenario 1, 2, or 3");
            std::process::exit(2);
        }
    }
}
