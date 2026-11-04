//! Supplied fixture: preserve this file. Length-bounded JSON over actual TCP.
use crate::{Reply, Request, gateway::Gateway};
use serde::{Serialize, de::DeserializeOwned};
use std::{io, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Semaphore,
    task::{JoinHandle, JoinSet},
    time::timeout,
};
const MAX_FRAME: usize = 128 * 1024;
async fn send<T: Serialize>(stream: &mut TcpStream, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_FRAME {
        return Err(io::Error::other("frame too large"));
    }
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await
}
async fn receive<T: DeserializeOwned>(stream: &mut TcpStream) -> io::Result<T> {
    let size = stream.read_u32().await? as usize;
    if size > MAX_FRAME {
        return Err(io::Error::other("frame too large"));
    }
    let mut bytes = vec![0; size];
    stream.read_exact(&mut bytes).await?;
    Ok(serde_json::from_slice(&bytes)?)
}
fn validate(req: &Request) -> io::Result<()> {
    if req.payload.len() > 64 * 1024 || req.path.len() > 2048 || req.delay_ms > 1000 {
        return Err(io::Error::other("fixture bounds exceeded"));
    }
    Ok(())
}
pub async fn forward(addr: SocketAddr, req: Request, attempt: usize) -> io::Result<u16> {
    let mut stream = TcpStream::connect(addr).await?;
    stream.set_nodelay(true)?;
    send(&mut stream, &(req, attempt)).await?;
    receive(&mut stream).await
}
pub async fn request(addr: SocketAddr, req: &Request) -> io::Result<Reply> {
    timeout(Duration::from_secs(10), async {
        let mut stream = TcpStream::connect(addr).await?;
        stream.set_nodelay(true)?;
        send(&mut stream, req).await?;
        receive(&mut stream).await
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "client fixture deadline"))?
}
pub struct Fixture {
    pub addr: SocketAddr,
    pub upstream: SocketAddr,
    pub arrivals: Arc<std::sync::atomic::AtomicUsize>,
    servers: Vec<JoinHandle<()>>,
}
impl Fixture {
    pub async fn start(service: Arc<Gateway>, gate: Option<Arc<Semaphore>>) -> io::Result<Self> {
        let upstream = TcpListener::bind("127.0.0.1:0").await?;
        let upstream_addr = upstream.local_addr()?;
        let gateway = TcpListener::bind("127.0.0.1:0").await?;
        let addr = gateway.local_addr()?;
        let arrivals = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = Arc::clone(&arrivals);
        let origin_task = tokio::spawn(async move {
            let mut children = JoinSet::new();
            let limit = Arc::new(Semaphore::new(128));
            loop {
                tokio::select! {
                    accepted=upstream.accept()=>{
                        let Ok((mut stream,_))=accepted else {break};
                        let Ok(slot)=Arc::clone(&limit).try_acquire_owned() else {continue};
                        let gate=gate.clone(); let observed=Arc::clone(&observed);
                        children.spawn(async move {
                            let _slot=slot;
                            let _=timeout(Duration::from_secs(4),async {
                                let (req,attempt):(Request,usize)=receive(&mut stream).await?;validate(&req)?;observed.fetch_add(1,std::sync::atomic::Ordering::SeqCst);
                                let work=async {
                                    if let Some(g)=gate { let permit=g.acquire().await.map_err(|_|io::Error::other("gate closed"))?;permit.forget(); }
                                    tokio::time::sleep(Duration::from_millis(req.delay_ms)).await;
                                    Ok::<(),io::Error>(())
                                };
                                let mut byte=[0];
                                tokio::select! { result=work=>{result?;}, _=stream.read(&mut byte)=>{return Ok::<(),io::Error>(());} }
                                send(&mut stream,&if req.fail_first&&attempt==0 {503u16}else{200u16}).await
                            }).await;
                        });
                    }, _=children.join_next(),if !children.is_empty()=>{}
                }
            }
        });
        let gateway_task = tokio::spawn(async move {
            let mut children = JoinSet::new();
            let limit = Arc::new(Semaphore::new(128));
            loop {
                tokio::select! {
                    accepted=gateway.accept()=>{
                        let Ok((mut stream,_))=accepted else {break};
                        let Ok(slot)=Arc::clone(&limit).try_acquire_owned() else {continue};
                        let service=Arc::clone(&service);
                        children.spawn(async move {
                            let _slot=slot;
                            let _=timeout(Duration::from_secs(8),async {
                                let req:Request=receive(&mut stream).await?;validate(&req)?;
                                let reply=service.handle(req,upstream_addr).await;
                                send(&mut stream,&reply).await
                            }).await;
                        });
                    }, _=children.join_next(),if !children.is_empty()=>{}
                }
            }
        });
        Ok(Self {
            addr,
            upstream: upstream_addr,
            arrivals,
            servers: vec![origin_task, gateway_task],
        })
    }
    pub async fn shutdown(mut self) {
        for task in &self.servers {
            task.abort();
        }
        for task in self.servers.drain(..) {
            let _ = task.await;
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        for task in &self.servers {
            task.abort();
        }
    }
}
