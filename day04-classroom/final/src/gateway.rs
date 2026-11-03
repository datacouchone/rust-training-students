use crate::{
    Reply, Request,
    policy::{self, Ledger, Router},
    telemetry::Buffer,
    transport,
};
use std::{
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::Mutex as AsyncMutex,
    time::{Instant, timeout_at},
};
#[derive(Default)]
pub struct Metrics {
    pub active: AtomicUsize,
    pub peak: AtomicUsize,
    pub attempts: AtomicUsize,
    pub retries: AtomicUsize,
}
struct Active(Arc<Metrics>);
impl Active {
    fn new(m: Arc<Metrics>) -> Self {
        let n = m.active.fetch_add(1, Ordering::SeqCst) + 1;
        m.peak.fetch_max(n, Ordering::SeqCst);
        m.attempts.fetch_add(1, Ordering::SeqCst);
        Self(m)
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::SeqCst);
    }
}
pub struct Gateway {
    pub router: AsyncMutex<Router>,
    pub ledger: Arc<Mutex<Ledger>>,
    pub retry_tokens: AtomicUsize,
    pub metrics: Arc<Metrics>,
    pub buffer: Mutex<Buffer>,
    pub budget: Duration,
}
impl Gateway {
    pub fn new(routes: usize, limit: usize, retries: usize, budget: Duration) -> Arc<Self> {
        Arc::new(Self {
            router: AsyncMutex::new(Router::new(routes)),
            ledger: Arc::new(Mutex::new(Ledger {
                used: 0,
                accepted: 0,
                limit,
            })),
            retry_tokens: AtomicUsize::new(retries),
            metrics: Arc::new(Metrics::default()),
            buffer: Mutex::new(Buffer {
                bytes: Vec::new(),
                retain: 4096,
            }),
            budget,
        })
    }
    // REPAIR deadline: callers supply the request start, including queue time.
    pub fn deadline(&self, _start: Instant) -> Instant {
        Instant::now() + self.budget
    }
    async fn attempt(
        self: Arc<Self>,
        upstream: SocketAddr,
        req: Request,
        attempt: usize,
    ) -> std::io::Result<u16> {
        let _active = Active::new(Arc::clone(&self.metrics));
        transport::forward(upstream, req, attempt).await
    }
    // REPAIR ownership: returning a timeout must leave no locally owned attempt running.
    async fn call(
        self: &Arc<Self>,
        upstream: SocketAddr,
        req: &Request,
        start: Instant,
    ) -> &'static str {
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(15 + req.id % 7)).await;
                if !policy::take_retry(&self.retry_tokens) {
                    return "retry-budget";
                }
                self.metrics.retries.fetch_add(1, Ordering::SeqCst);
            }
            let task = tokio::spawn(Arc::clone(self).attempt(upstream, req.clone(), attempt));
            match timeout_at(self.deadline(start), task).await {
                Ok(Ok(Ok(200))) => return "ok",
                Ok(Ok(Ok(503))) => continue,
                Ok(_) => return "upstream-error",
                Err(_) => return "timeout",
            }
        }
        "attempt-limit"
    }
    pub async fn handle(self: &Arc<Self>, req: Request, upstream: SocketAddr) -> Reply {
        let start = Instant::now();
        let mut reply = Reply {
            status: "rejected".into(),
            pool: None,
            queue_us: 0,
            route_us: 0,
            upstream_us: 0,
            total_us: 0,
        };
        // Transport has already bounded the frame. Admission owns logical payload bytes.
        let Some(_lease) = policy::admit(&self.ledger, req.payload.len()) else {
            return reply;
        };
        // REPAIR synchronization: separate policy access from upstream waiting.
        let router = self.router.lock().await;
        reply.queue_us = start.elapsed().as_micros() as u64;
        let route = Instant::now();
        reply.pool = router.route(req.tenant, &req.path);
        reply.route_us = route.elapsed().as_micros() as u64;
        if reply.pool.is_none() {
            reply.status = "no-route".into();
            return reply;
        }
        let io = Instant::now();
        reply.status = self.call(upstream, &req, start).await.into();
        reply.upstream_us = io.elapsed().as_micros() as u64;
        drop(router);
        let mut telemetry = self.buffer.lock().unwrap();
        telemetry.bytes.extend_from_slice(req.payload.as_bytes());
        telemetry.flush();
        reply.total_us = start.elapsed().as_micros() as u64;
        reply
    }
}
