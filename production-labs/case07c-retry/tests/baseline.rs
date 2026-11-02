use case07c_retry::*;
use std::time::Duration;
#[tokio::test(start_paused = true)]
async fn supplied_transport_observes_one_committed_effect() {
    let t = Scripted::new(vec![Plan {
        after: Duration::from_millis(3),
        response: Response::Success,
    }]);
    assert!(matches!(t.send(Some("demo-key")).await, Response::Success));
    assert_eq!(
        t.observe(),
        Observation {
            started: 1,
            active: 0,
            commits: 1,
            keys: vec![Some("demo-key".into())]
        }
    );
    assert_eq!(RetryBudget::new(4).remaining(), 4);
}
