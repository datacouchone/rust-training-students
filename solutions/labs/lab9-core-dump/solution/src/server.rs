//! Connection acceptance and per-frame handling.
//!
//! This module is also where all three injected faults live, each gated
//! on `state.fault` so the healthy path (`state.fault == None`) never
//! touches any of this fault-injection code.

use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tracing::{Instrument, info, warn};

use crate::fault::Fault;
use crate::frame::{Metric, parse_metric_frame, read_frame, write_frame};
use crate::state::AppState;

/// Accept connections forever, spawning one task per connection.
///
/// If `FAULT=deadlock`, arms the AB-BA mutex deadlock pair once at
/// startup (see [`arm_deadlock`]) -- the bug is "already in the code",
/// independent of any particular request, exactly like a real deadlock
/// that two background tasks stumble into under load.
pub async fn run(listener: TcpListener, state: AppState) -> std::io::Result<()> {
    let local_addr = listener.local_addr()?;
    info!(fault = ?state.fault, %local_addr, "lab9 metrics server listening");

    if state.fault == Some(Fault::Deadlock) {
        warn!("FAULT=deadlock active: arming AB-BA mutex deadlock pair now");
        arm_deadlock(state.clone());
    }

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        let conn_id = state.next_conn_id();
        let conn_state = state.clone();
        info!(conn_id, %peer_addr, "accepted connection");
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, conn_state, conn_id).await {
                warn!(conn_id, error = %e, "connection task ended with an I/O error");
            }
        });
    }
}

/// Handle one connection end-to-end: read frames until EOF or an I/O
/// error, answering each with `OK` or `ERR <reason>`.
#[tracing::instrument(skip(stream, state))]
pub async fn handle_connection<S>(stream: S, state: AppState, conn_id: u64) -> std::io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    info!("connection opened");
    {
        let mut registry = state.registry.lock().await;
        registry.push(conn_id);
    } // guard dropped here -- never held across an .await

    let (mut reader, mut writer) = tokio::io::split(stream);
    let mut frame_no: u64 = 0;

    loop {
        let frame = match read_frame(&mut reader).await {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                info!("peer closed connection (EOF)");
                break;
            }
            Err(e) => {
                warn!(error = %e, "frame read error; closing connection");
                break;
            }
        };
        frame_no += 1;
        let response = process_frame(conn_id, frame_no, &frame, &state).await;
        write_frame(&mut writer, response.as_bytes()).await?;
    }

    {
        let mut registry = state.registry.lock().await;
        registry.retain(|&id| id != conn_id);
    } // guard dropped here -- never held across an .await
    info!("connection closed");
    Ok(())
}

/// Parse and record one frame, producing the response frame's text body.
///
/// Also where the `task_leak` and `panic` faults are triggered, gated on
/// `state.fault`.
#[tracing::instrument(skip(bytes, state))]
async fn process_frame(conn_id: u64, frame_no: u64, bytes: &[u8], state: &AppState) -> String {
    if state.fault == Some(Fault::TaskLeak) {
        spawn_leaked_task(conn_id, frame_no);
    }

    let parsed = parse_metric_frame(bytes);

    if state.fault == Some(Fault::Panic) {
        // FAULT INJECTION (panic): correct code always matches on `parsed`
        // and answers ERR for bad input -- see the `else` branch below.
        // This `.unwrap()` is the bug: a malformed frame panics this
        // connection's task instead of being rejected gracefully.
        let metric = parsed.unwrap();
        record_metric(state, metric).await;
        "OK".to_string()
    } else {
        match parsed {
            Ok(metric) => {
                record_metric(state, metric).await;
                "OK".to_string()
            }
            Err(e) => {
                warn!(error = %e, "rejecting malformed frame");
                format!("ERR {e}")
            }
        }
    }
}

async fn record_metric(state: &AppState, metric: Metric) {
    let mut guard = state.metrics.lock().await;
    guard.insert(metric.name, metric.value);
} // guard dropped here -- never held across an .await

/// FAULT INJECTION (task_leak): spawn a task that parks forever, never
/// joined, aborted, or otherwise tracked. One is spawned per frame while
/// this fault is active, so the live task count -- visible in
/// `tokio-console`'s task list -- grows without bound and never shrinks.
fn spawn_leaked_task(conn_id: u64, frame_no: u64) {
    tokio::spawn(
        async move {
            tracing::info!("leaked task started; it will never terminate");
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        }
        .instrument(tracing::info_span!("leaked_task", conn_id, frame_no)),
    );
}

/// FAULT INJECTION (deadlock): the classic AB-BA pair.
///
/// Task A locks `metrics`, then -- while still holding that guard --
/// awaits before locking `registry`. Task B locks `registry`, then --
/// while still holding that guard -- awaits before locking `metrics`.
/// Once both tasks have taken their first lock, each blocks forever
/// waiting on the lock the other is holding. Because every frame's
/// `record_metric` and every connection's open/close also need these same
/// two locks, the whole server wedges shortly after this is armed.
fn arm_deadlock(state: AppState) {
    let a1 = state.metrics.clone();
    let b1 = state.registry.clone();
    tokio::spawn(
        async move {
            let _guard_a = a1.lock().await;
            tracing::info!("task_a: locked metrics; sleeping before locking registry");
            tokio::time::sleep(Duration::from_millis(100)).await;
            tracing::info!("task_a: attempting registry lock");
            let _guard_b = b1.lock().await; // never returns under this fault
            tracing::info!("task_a: locked registry (unreachable while deadlocked)");
        }
        .instrument(tracing::info_span!("deadlock_task_a")),
    );

    let a2 = state.metrics.clone();
    let b2 = state.registry.clone();
    tokio::spawn(
        async move {
            let _guard_b = b2.lock().await;
            tracing::info!("task_b: locked registry; sleeping before locking metrics");
            tokio::time::sleep(Duration::from_millis(100)).await;
            tracing::info!("task_b: attempting metrics lock");
            let _guard_a = a2.lock().await; // never returns under this fault
            tracing::info!("task_b: locked metrics (unreachable while deadlocked)");
        }
        .instrument(tracing::info_span!("deadlock_task_b")),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{read_frame, write_frame};

    /// FAULT unset -> healthy behavior: well-formed frames are recorded
    /// and answered `OK`; malformed frames are answered `ERR ...` and the
    /// connection stays alive (no panic). Entirely in-process over a
    /// `tokio::io::duplex` pair -- no real socket, no sleeps used as
    /// synchronization.
    #[tokio::test]
    async fn test_healthy_path_no_fault() {
        let (mut client, server_stream) = tokio::io::duplex(4096);
        let state = AppState::new(None); // FAULT unset

        let server_task = tokio::spawn(handle_connection(server_stream, state.clone(), 1));

        write_frame(&mut client, b"METRIC cpu.load 0.42")
            .await
            .expect("write should succeed");
        let response = read_frame(&mut client)
            .await
            .expect("read should succeed")
            .expect("server should respond");
        assert_eq!(response, b"OK");

        // Malformed input on the healthy path must be answered ERR, never
        // panic the connection.
        write_frame(&mut client, b"METRIC cpu.load not_a_number")
            .await
            .expect("write should succeed");
        let response = read_frame(&mut client)
            .await
            .expect("read should succeed")
            .expect("server should respond");
        assert!(
            response.starts_with(b"ERR"),
            "expected ERR response, got {:?}",
            String::from_utf8_lossy(&response)
        );

        drop(client); // EOF -> handler loop exits cleanly
        let result = server_task
            .await
            .expect("connection task should not panic under FAULT unset");
        assert!(result.is_ok());

        let metrics = state.metrics.lock().await;
        assert_eq!(metrics.get("cpu.load"), Some(&0.42));
    }
}
