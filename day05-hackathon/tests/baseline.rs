use day05_hackathon::{
    boundary::encode,
    fixture::{Server, send},
    worker::Service,
};
use std::time::Duration;
#[tokio::test]
async fn native_tcp_smoke() {
    let s = Service::new(2, 2, 4);
    let server = Server::start(s, Duration::ZERO, Duration::from_secs(1))
        .await
        .unwrap();
    assert_eq!(
        send(server.address, &encode(1, 1, 10, "/a")).await.unwrap(),
        0
    );
}
