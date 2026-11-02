use case07_worker::*;
#[tokio::test(start_paused = true)]
async fn virtual_clock_and_probe_are_available() {
    let p = Probe::default();
    assert_eq!((p.active(), p.peak()), (0, 0));
    let before = tokio::time::Instant::now();
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    assert_eq!(before.elapsed(), std::time::Duration::from_secs(10));
}
