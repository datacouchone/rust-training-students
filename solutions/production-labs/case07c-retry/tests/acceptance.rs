use case07c_retry::*;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::watch,
    time::{self, Instant},
};
fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}
fn policy() -> Policy {
    Policy {
        max_attempts: 3,
        total_timeout: ms(100),
        base_backoff: ms(10),
        max_backoff: ms(40),
        jitter: |cap| cap,
    }
}
fn op(key: bool) -> Operation {
    Operation {
        idempotency_key: key.then(|| "stable-request-42".into()),
    }
}
fn plan(response: Response) -> Plan {
    Plan {
        after: ms(1),
        response,
    }
}
fn transient() -> Response {
    Response::RetryableBeforeCommit {
        retry_after: Duration::ZERO,
    }
}
async fn run(t: Scripted, p: Policy, b: Arc<RetryBudget>, key: bool) -> Result<Receipt, Failure> {
    let (_tx, rx) = watch::channel(false);
    execute(op(key), p, b, t, rx).await
}
#[tokio::test(start_paused = true)]
async fn first_attempt_is_free_and_success_is_not_retried() {
    let t = Scripted::new(vec![plan(Response::Success)]);
    let b = RetryBudget::new(0);
    assert_eq!(
        run(t.clone(), policy(), b.clone(), false).await,
        Ok(Receipt { attempts: 1 })
    );
    assert_eq!(b.remaining(), 0);
    assert_eq!(t.observe().started, 1);
    assert_eq!(t.observe().active, 0);
}
#[tokio::test(start_paused = true)]
async fn only_extra_attempts_consume_shared_tokens() {
    let t = Scripted::new(vec![plan(transient()), plan(Response::Success)]);
    let b = RetryBudget::new(1);
    assert_eq!(
        run(t.clone(), policy(), b.clone(), false).await,
        Ok(Receipt { attempts: 2 })
    );
    assert_eq!(b.remaining(), 0);
    assert_eq!(t.observe().commits, 1);
}
#[tokio::test(start_paused = true)]
async fn ambiguous_non_idempotent_work_is_never_retried() {
    let t = Scripted::new(vec![
        plan(Response::CommitThenLoseReply),
        plan(Response::Success),
    ]);
    let b = RetryBudget::new(5);
    assert_eq!(
        run(t.clone(), policy(), b.clone(), false).await,
        Err(Failure {
            reason: Stop::Uncertain,
            attempts: 1,
            may_have_committed: true
        })
    );
    assert_eq!(t.observe().started, 1);
    assert_eq!(t.observe().commits, 1);
    assert_eq!(b.remaining(), 5);
}
#[tokio::test(start_paused = true)]
async fn keyed_retry_reuses_identity_and_does_not_duplicate_effect() {
    let t = Scripted::new(vec![
        plan(Response::CommitThenLoseReply),
        plan(Response::Success),
    ]);
    assert_eq!(
        run(t.clone(), policy(), RetryBudget::new(2), true).await,
        Ok(Receipt { attempts: 2 })
    );
    assert_eq!(t.observe().keys, vec![Some("stable-request-42".into()); 2]);
    assert_eq!(t.observe().commits, 1);
}
#[tokio::test(start_paused = true)]
async fn absolute_deadline_includes_backoff_and_retry_after() {
    let t = Scripted::new(vec![
        plan(Response::RetryableBeforeCommit {
            retry_after: ms(200),
        }),
        plan(Response::Success),
    ]);
    let b = RetryBudget::new(3);
    let start = Instant::now();
    assert_eq!(
        run(t.clone(), policy(), b.clone(), false).await,
        Err(Failure {
            reason: Stop::Deadline,
            attempts: 1,
            may_have_committed: false
        })
    );
    assert_eq!(Instant::now() - start, ms(100));
    assert_eq!(t.observe().started, 1);
    assert_eq!(b.remaining(), 3);
}
#[tokio::test(start_paused = true)]
async fn deadline_does_not_restart_on_the_second_attempt() {
    let t = Scripted::new(vec![
        Plan {
            after: ms(60),
            response: transient(),
        },
        Plan {
            after: ms(60),
            response: Response::Success,
        },
    ]);
    let start = Instant::now();
    assert_eq!(
        run(t.clone(), policy(), RetryBudget::new(1), false).await,
        Err(Failure {
            reason: Stop::Deadline,
            attempts: 2,
            may_have_committed: true
        })
    );
    assert_eq!(Instant::now() - start, ms(100));
    assert_eq!(t.observe().active, 0);
}
#[tokio::test(start_paused = true)]
async fn cancellation_during_backoff_does_not_spend_another_token() {
    let t = Scripted::new(vec![plan(transient()), plan(Response::Success)]);
    let b = RetryBudget::new(2);
    let (tx, rx) = watch::channel(false);
    let handle = tokio::spawn(execute(op(false), policy(), b.clone(), t.clone(), rx));
    tokio::task::yield_now().await;
    time::advance(ms(2)).await;
    tokio::task::yield_now().await;
    assert_eq!(t.observe().started, 1);
    assert_eq!(t.observe().active, 0);
    tx.send(true).unwrap();
    assert_eq!(
        handle.await.unwrap(),
        Err(Failure {
            reason: Stop::Cancelled,
            attempts: 1,
            may_have_committed: false
        })
    );
    assert_eq!(b.remaining(), 2);
    assert_eq!(t.observe().started, 1);
}
#[tokio::test(start_paused = true)]
async fn cancellation_drops_in_flight_work_and_preserves_uncertainty() {
    let t = Scripted::new(vec![Plan {
        after: ms(80),
        response: Response::Success,
    }]);
    let (tx, rx) = watch::channel(false);
    let handle = tokio::spawn(execute(
        op(false),
        policy(),
        RetryBudget::new(1),
        t.clone(),
        rx,
    ));
    tokio::task::yield_now().await;
    assert_eq!(t.observe().active, 1);
    tx.send(true).unwrap();
    assert_eq!(
        handle.await.unwrap(),
        Err(Failure {
            reason: Stop::Cancelled,
            attempts: 1,
            may_have_committed: true
        })
    );
    assert_eq!(t.observe().active, 0);
}
#[tokio::test(start_paused = true)]
async fn dropping_parent_future_releases_in_flight_transport() {
    let t = Scripted::new(vec![Plan {
        after: ms(80),
        response: Response::Success,
    }]);
    let (_tx, rx) = watch::channel(false);
    let mut future = Box::pin(execute(
        op(false),
        policy(),
        RetryBudget::new(1),
        t.clone(),
        rx,
    ));
    assert!(time::timeout(ms(1), &mut future).await.is_err());
    assert_eq!(t.observe().active, 1);
    drop(future);
    assert_eq!(t.observe().active, 0);
}
#[tokio::test(start_paused = true)]
async fn already_cancelled_admits_nothing() {
    let t = Scripted::new(vec![plan(Response::Success)]);
    let (_tx, rx) = watch::channel(true);
    assert_eq!(
        execute(op(false), policy(), RetryBudget::new(0), t.clone(), rx).await,
        Err(Failure {
            reason: Stop::Cancelled,
            attempts: 0,
            may_have_committed: false
        })
    );
    assert_eq!(t.observe().started, 0);
}
#[tokio::test(start_paused = true)]
async fn concurrent_requests_share_one_retry_budget() {
    let a = Scripted::new(vec![plan(transient()), plan(Response::Success)]);
    let b = Scripted::new(vec![plan(transient()), plan(Response::Success)]);
    let budget = RetryBudget::new(1);
    let (ra, rb) = tokio::join!(
        run(a.clone(), policy(), budget.clone(), false),
        run(b.clone(), policy(), budget.clone(), false)
    );
    assert_eq!([ra, rb].iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        [ra, rb]
            .iter()
            .filter(|r| matches!(
                r,
                Err(Failure {
                    reason: Stop::RetryBudget,
                    ..
                })
            ))
            .count(),
        1
    );
    assert_eq!(a.observe().started + b.observe().started, 3);
    assert_eq!(budget.remaining(), 0);
}
#[tokio::test(start_paused = true)]
async fn attempt_cap_and_permanent_failure_stop_immediately() {
    let mut p = policy();
    p.max_attempts = 1;
    let b = RetryBudget::new(2);
    let t = Scripted::new(vec![plan(transient()), plan(Response::Success)]);
    assert_eq!(
        run(t.clone(), p, b.clone(), false)
            .await
            .unwrap_err()
            .reason,
        Stop::AttemptsExhausted
    );
    assert_eq!(t.observe().started, 1);
    assert_eq!(b.remaining(), 2);
    let t = Scripted::new(vec![plan(Response::Permanent), plan(Response::Success)]);
    assert_eq!(
        run(t.clone(), policy(), b, false).await.unwrap_err().reason,
        Stop::Permanent
    );
    assert_eq!(t.observe().started, 1);
}
#[tokio::test(start_paused = true)]
async fn later_failure_does_not_erase_earlier_commit_uncertainty() {
    let t = Scripted::new(vec![
        plan(Response::CommitThenLoseReply),
        plan(Response::Permanent),
    ]);
    let error = run(t, policy(), RetryBudget::new(1), true)
        .await
        .unwrap_err();
    assert_eq!(error.reason, Stop::Permanent);
    assert!(error.may_have_committed);
    let t = Scripted::new(vec![plan(Response::CommitThenLoseReply)]);
    let error = run(t, policy(), RetryBudget::new(0), true)
        .await
        .unwrap_err();
    assert_eq!(error.reason, Stop::RetryBudget);
    assert!(error.may_have_committed);
}
#[test]
fn backoff_is_capped_jitter_is_clamped_and_retry_after_is_a_floor() {
    let mut p = policy();
    p.jitter = |cap| cap / 2;
    assert_eq!(retry_delay(p, 1, Duration::ZERO), ms(5));
    assert_eq!(retry_delay(p, 2, Duration::ZERO), ms(10));
    assert_eq!(retry_delay(p, 20, Duration::ZERO), ms(20));
    p.jitter = |_| Duration::MAX;
    assert_eq!(retry_delay(p, 20, ms(60)), ms(60));
    assert_eq!(retry_delay(p, 1, Duration::ZERO), ms(10));
}
#[tokio::test(start_paused = true)]
async fn invalid_policy_or_empty_key_starts_no_transport() {
    let t = Scripted::new(vec![plan(Response::Success)]);
    let mut p = policy();
    p.max_attempts = 0;
    assert_eq!(
        run(t.clone(), p, RetryBudget::new(2), false)
            .await
            .unwrap_err()
            .reason,
        Stop::InvalidPolicy
    );
    let (_tx, rx) = watch::channel(false);
    assert_eq!(
        execute(
            Operation {
                idempotency_key: Some(String::new())
            },
            policy(),
            RetryBudget::new(2),
            t.clone(),
            rx
        )
        .await
        .unwrap_err()
        .reason,
        Stop::InvalidPolicy
    );
    assert_eq!(t.observe().started, 0);
}
