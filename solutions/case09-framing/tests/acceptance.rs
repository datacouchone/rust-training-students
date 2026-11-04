use case09_incident::*;
use std::{io::ErrorKind, time::Duration};
use tokio::io::{AsyncWriteExt, duplex};
#[tokio::test(start_paused = true)]
async fn cancellation_after_every_byte_preserves_frame() {
    let bytes = frame(b"12:100");
    for split in 1..bytes.len() {
        let (mut tx, mut rx) = duplex(64);
        let mut d = Decoder::default();
        tx.write_all(&bytes[..split]).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(1), d.next(&mut rx))
                .await
                .is_err()
        );
        assert_eq!(d.buffered(), split);
        tx.write_all(&bytes[split..]).await.unwrap();
        drop(tx);
        assert_eq!(d.next(&mut rx).await.unwrap(), Some(b"12:100".to_vec()));
    }
}
#[tokio::test]
async fn back_to_back_empty_and_clean_eof() {
    let (mut tx, mut rx) = duplex(64);
    tx.write_all(&frame(b"")).await.unwrap();
    tx.write_all(&frame(b"1:9")).await.unwrap();
    drop(tx);
    let mut d = Decoder::default();
    assert_eq!(d.next(&mut rx).await.unwrap(), Some(vec![]));
    assert_eq!(d.next(&mut rx).await.unwrap(), Some(b"1:9".to_vec()));
    assert_eq!(d.next(&mut rx).await.unwrap(), None);
}
#[tokio::test]
async fn rejects_length_before_body_allocation_and_poisoned_retry() {
    let (mut tx, mut rx) = duplex(64);
    tx.write_all(&u32::MAX.to_be_bytes()).await.unwrap();
    let mut d = Decoder::default();
    assert_eq!(
        d.next(&mut rx).await.unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(d.buffered(), 4);
    assert_eq!(
        tokio::time::timeout(Duration::from_millis(100), d.next(&mut rx))
            .await
            .expect("poisoned stream must fail immediately")
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
}
#[tokio::test]
async fn every_truncated_frame_is_distinct_from_clean_eof() {
    let bytes = frame(b"1:2");
    for n in 1..bytes.len() {
        let (mut tx, mut rx) = duplex(64);
        tx.write_all(&bytes[..n]).await.unwrap();
        drop(tx);
        assert_eq!(
            Decoder::default().next(&mut rx).await.unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );
    }
}
#[test]
fn accounting_is_transactional_on_overflow() {
    let mut g = Gateway::new(4);
    g.apply(format!("1:{}", u64::MAX).as_bytes()).unwrap();
    assert_eq!(g.apply(b"1:1"), Err(Reject::Overflow));
    assert_eq!(
        g.accounts()[&1],
        Account {
            accepted: 1,
            bytes: u64::MAX
        }
    );
}
#[test]
fn invalid_frames_do_not_allocate_tenants() {
    let mut g = Gateway::new(1);
    for b in [&b"1:bad"[..], b"bad", b"\xff", b"1:2:3"] {
        assert!(g.apply(b).is_err());
    }
    assert!(g.accounts().is_empty());
    g.apply(b"1:2").unwrap();
    assert_eq!(g.apply(b"2:3"), Err(Reject::Cardinality));
    assert_eq!(
        g.apply(b"1:3").unwrap(),
        Account {
            accepted: 2,
            bytes: 5
        }
    );
}
