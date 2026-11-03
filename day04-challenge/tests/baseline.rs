use day04_challenge::{
    Request,
    gateway::Gateway,
    transport::{Fixture, request},
};
use std::{sync::Arc, time::Duration};
#[tokio::test]
async fn one_request_crosses_two_real_tcp_connections() {
    let service = Gateway::new(32, 65536, 4, Duration::from_secs(2));
    let f = Fixture::start(Arc::clone(&service), None).await.unwrap();
    let r = request(f.addr, &Request::sample(1)).await.unwrap();
    assert_eq!(r.status, "ok");
    assert_eq!(r.pool, Some(17));
    assert_eq!(service.ledger.lock().unwrap().used, 0);
    f.shutdown().await;
}
