//! Supplied bounded native-TCP fixture. Keep this file unchanged during repairs.
use crate::{
    boundary::{Untrusted, Validated},
    codec::{Decoder, frame},
    ledger::Outcome,
    worker::{Error, Service},
};
use std::{io, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinSet,
    time::{Instant, timeout},
};
pub struct Server {
    pub address: SocketAddr,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Server {
    pub async fn start(
        service: Arc<Service>,
        delay: Duration,
        budget: Duration,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let task = tokio::spawn(async move {
            let mut children = JoinSet::new();
            loop {
                tokio::select! {
                 accepted=listener.accept()=>{
                  let Ok((mut socket,_))=accepted else{break};
                  if children.len()>=64 {drop(socket);continue}
                  let service=Arc::clone(&service);
                  children.spawn(async move {
                   let _=timeout(Duration::from_secs(3),async {
                    let body=Decoder::default().next(&mut socket).await?.ok_or_else(||io::Error::other("empty connection"))?;
                    let event=Validated::try_from(Untrusted(body)).map_err(|_|io::Error::other("invalid event"))?;
                    let code=match service.submit(event,Instant::now()+budget,delay).await {Ok(Outcome::Applied)=>0,Ok(Outcome::Replay)=>1,Err(Error::Timeout)=>3,Err(_)=>2};
                    socket.write_all(&[code]).await
                   }).await;
                  });
                 }
                 _=children.join_next(),if !children.is_empty()=>{}
                }
            }
        });
        Ok(Self { address, task })
    }
}
pub async fn send(address: SocketAddr, body: &[u8]) -> io::Result<u8> {
    timeout(Duration::from_secs(4), async {
        let mut socket = TcpStream::connect(address).await?;
        socket.write_all(&frame(body)).await?;
        socket.read_u8().await
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "fixture client watchdog"))?
}
