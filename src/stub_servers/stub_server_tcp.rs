use crate::RnpStubServerConfig;
use futures_intrusive::sync::ManualResetEvent;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::Interest;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

pub struct StubServerTcp {
    config: Arc<RnpStubServerConfig>,
    stop_event: Arc<ManualResetEvent>,

    next_conn_id: u32,
    conn_stats_map: HashMap<u32, Arc<Mutex<StubServerTcpConnectionStats>>>,
}

impl StubServerTcp {
    pub fn run_new(config: RnpStubServerConfig, stop_event: Arc<ManualResetEvent>) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
        return tokio::spawn(async move {
            let mut server = StubServerTcp::new(config, stop_event);
            return server.run().await;
        });
    }

    fn new(config: RnpStubServerConfig, stop_event: Arc<ManualResetEvent>) -> StubServerTcp {
        return StubServerTcp { config: Arc::new(config), stop_event, next_conn_id: 0, conn_stats_map: HashMap::new() };
    }

    async fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(self.config.server_address).await?;

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, peer_addr)) => {
                            self.handle_new_connection(stream, peer_addr).await
                        },
                        Err(e) => {
                            println!("Failed to accept new connection. Exit: Error = {}", e);
                            break;
                        }
                    }
                }
                _ = self.stop_event.wait() => { return Ok(()); }
            }
        }

        return Ok(());
    }

    async fn handle_new_connection(&mut self, stream: TcpStream, peer_addr: SocketAddr) {
        println!("New connection received: Remote = {}", peer_addr);

        let stream_config = self.config.clone();
        let stream_stop_event = self.stop_event.clone();

        let conn_id = self.next_conn_id;
        self.next_conn_id += 1;

        let conn_stats = Arc::new(Mutex::new(StubServerTcpConnectionStats::new()));
        self.conn_stats_map.insert(conn_id, conn_stats.clone());

        tokio::spawn(async move {
            let mut worker = StubServerTcpConnection::new(conn_id, stream_config, stream, peer_addr, conn_stats);
            tokio::select! {
                _ = worker.run() => { return; }
                _ = stream_stop_event.wait() => { return; }
            }
        });
    }
}

struct StubServerTcpConnection {
    id: u32,
    config: Arc<RnpStubServerConfig>,
    stream: TcpStream,
    remote_address: SocketAddr,
    read_buf: Vec<u8>,
    conn_stats: Arc<Mutex<StubServerTcpConnectionStats>>,
}

impl StubServerTcpConnection {
    fn new(
        id: u32,
        config: Arc<RnpStubServerConfig>,
        stream: TcpStream,
        remote_address: SocketAddr,
        conn_stats: Arc<Mutex<StubServerTcpConnectionStats>>,
    ) -> StubServerTcpConnection {
        return StubServerTcpConnection { id, config, stream, remote_address, read_buf: vec![0; 4096], conn_stats };
    }

    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let result = self.run_loop().await;
        self.conn_stats.lock().unwrap().is_alive = false;
        return result;
    }

    async fn run_loop(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let ready = self.stream.ready(Interest::READABLE | Interest::WRITABLE).await?;

            if ready.is_readable() {
                self.on_connection_read().await?;
            }

            if ready.is_writable() {
                self.on_connection_write().await?;
            }
        }
    }

    async fn on_connection_read(&mut self) -> Result<(), Box<dyn Error>> {
        match self.stream.try_read(&mut self.read_buf) {
            Ok(n) => {
                self.conn_stats.lock().unwrap().bytes_read += n;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => {
                println!("Error found in connection to {}, connection closed: Error = {}", self.remote_address, e);
                return Err(e.into());
            }
        }

        return Ok(());
    }

    async fn on_connection_write(&mut self) -> Result<(), Box<dyn Error>> {
        match self.stream.try_write(b"hello world") {
            Ok(n) => {
                self.conn_stats.lock().unwrap().bytes_write += n;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => {
                println!("Error found in connection to {}, connection closed: Error = {}", self.remote_address, e);
                return Err(e.into());
            }
        }

        return Ok(());
    }
}

#[derive(Debug, Clone, PartialEq)]
struct StubServerTcpConnectionStats {
    pub is_alive: bool,
    pub bytes_read: usize,
    pub bytes_write: usize,
}

impl StubServerTcpConnectionStats {
    pub fn new() -> StubServerTcpConnectionStats {
        return StubServerTcpConnectionStats { is_alive: true, bytes_read: 0, bytes_write: 0 };
    }

    pub fn clone_and_clear(&mut self) -> StubServerTcpConnectionStats {
        let stats = self.clone();
        self.clear();
        return stats;
    }

    pub fn clear(&mut self) {
        self.bytes_write = 0;
        self.bytes_read = 0;
    }
}
