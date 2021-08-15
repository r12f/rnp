use crate::RnpStubServerConfig;
use futures_intrusive::sync::ManualResetEvent;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::Interest;
use std::io;

pub struct StubServerTcp {
    config: Arc<RnpStubServerConfig>,
    stop_event: Arc<ManualResetEvent>,
}

impl StubServerTcp {
    pub fn run_new(config: RnpStubServerConfig, stop_event: Arc<ManualResetEvent>) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
        return tokio::spawn(async move {
            let mut server = StubServerTcp { config: Arc::new(config), stop_event: stop_event.clone() };

            tokio::select! {
                res = server.run() => { return res; }
                _ = stop_event.wait() => { return Ok(()); }
            }
        });
    }

    async fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(self.config.server_address).await?;

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => self.handle_new_connection(stream, peer_addr).await,
                Err(e) => {
                    println!("Failed to accept new connection. Exit: Error = {}", e);
                    break;
                }
            }
        }

        return Ok(());
    }

    async fn handle_new_connection(&self, stream: TcpStream, peer_addr: SocketAddr) {
        println!("New connection received: Remote = {}", peer_addr);

        let stream_config = self.config.clone();
        let stream_stop_event = self.stop_event.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = StubServerTcp::handle_connection_worker(stream, stream_config) => { return; }
                _ = stream_stop_event.wait() => { return; }
            }
        });
    }

    async fn handle_connection_worker(stream: TcpStream, _: Arc<RnpStubServerConfig>) -> Result<(), Box<dyn Error>> {
        loop {
            let ready = stream.ready(Interest::READABLE | Interest::WRITABLE).await?;

            if ready.is_readable() {
                let mut data = vec![0; 1024];
                match stream.try_read(&mut data) {
                    Ok(n) => {
                        println!("read {} bytes", n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        println!("Error found in connection to {}, connection closed: Error = {}", stream.peer_addr()?, e);
                        return Err(e.into());
                    }
                }

            }

            if ready.is_writable() {
                match stream.try_write(b"hello world") {
                    Ok(n) => {
                        println!("write {} bytes", n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue
                    }
                    Err(e) => {
                        println!("Error found in connection to {}, connection closed: Error = {}", stream.peer_addr()?, e);
                        return Err(e.into());
                    }
                }
            }
        }
    }
}
