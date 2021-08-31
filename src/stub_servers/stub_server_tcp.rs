use crate::RnpStubServerConfig;
use futures_intrusive::sync::ManualResetEvent;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::Instant;

pub struct StubServerTcp {
    config: Arc<RnpStubServerConfig>,
    stop_event: Arc<ManualResetEvent>,
    server_started_event: Arc<ManualResetEvent>,

    next_conn_id: u32,
    conn_stats_map: HashMap<u32, Arc<Mutex<StubServerTcpConnectionStats>>>,
}

impl StubServerTcp {
    #[tracing::instrument(name = "Start running new TCP stub server", level = "debug", skip(stop_event))]
    pub fn run_new(
        config: RnpStubServerConfig,
        stop_event: Arc<ManualResetEvent>,
        server_started_event: Arc<ManualResetEvent>,
    ) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
        return tokio::spawn(async move {
            let mut server = StubServerTcp::new(config, stop_event, server_started_event.clone());

            // In case server started failed, we always signal server started event here to keep it safe.
            let result = server.run().await;
            server_started_event.set();

            return result;
        });
    }

    #[tracing::instrument(name = "Creating TCP stub server", level = "debug", skip(stop_event))]
    fn new(config: RnpStubServerConfig, stop_event: Arc<ManualResetEvent>, server_started_event: Arc<ManualResetEvent>) -> StubServerTcp {
        return StubServerTcp { config: Arc::new(config), stop_event, server_started_event, next_conn_id: 0, conn_stats_map: HashMap::new() };
    }

    #[tracing::instrument(name = "Running TCP stub server loop", level = "debug", skip(self))]
    async fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(self.config.server_address).await?;
        self.server_started_event.set();

        println!("Rnp {} server started successfully at {}.", self.config.protocol, self.config.server_address);

        let mut next_report_time = Instant::now();
        loop {
            tokio::select! {
                // New connection arrived.
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

                // Report interval reached
                _ = tokio::time::sleep_until(next_report_time) => {
                    self.report_and_reset_conn_stats();
                    next_report_time += self.config.report_interval;
                }

                // Stopped
                _ = self.stop_event.wait() => {
                    break;
                }
            }
        }

        return Ok(());
    }

    #[tracing::instrument(name = "New connection received", level = "debug", skip(self))]
    async fn handle_new_connection(&mut self, stream: TcpStream, peer_addr: SocketAddr) {
        println!("New connection received: Remote = {}", peer_addr);

        if self.config.close_on_accept {
            self.close_connection_on_accept(stream, peer_addr).await;
            return;
        }

        self.start_connection_worker(stream, peer_addr).await;
    }

    #[tracing::instrument(name = "Close connection on accept", level = "debug", skip(self))]
    async fn close_connection_on_accept(&mut self, mut stream: TcpStream, peer_addr: SocketAddr) {
        let _ = stream.shutdown().await;
        println!("Connection closed on accept: Remote = {}", peer_addr);
    }

    #[tracing::instrument(name = "Starting new connection worker", level = "debug", skip(self))]
    async fn start_connection_worker(&mut self, stream: TcpStream, peer_addr: SocketAddr) {
        let stream_config = self.config.clone();
        let stream_stop_event = self.stop_event.clone();

        let conn_id = self.next_conn_id;
        self.next_conn_id += 1;

        let conn_stats = Arc::new(Mutex::new(StubServerTcpConnectionStats::new(&peer_addr)));
        self.conn_stats_map.insert(conn_id, conn_stats.clone());

        tokio::spawn(async move {
            let mut worker = StubServerTcpConnection::new(conn_id, stream_config, stream, peer_addr, conn_stats);
            tokio::select! {
                _ = worker.run() => { return; }
                _ = stream_stop_event.wait() => { return; }
            }
        });
    }

    #[tracing::instrument(name = "Report and reset connection stats", level = "debug", skip(self))]
    fn report_and_reset_conn_stats(&mut self) {
        if self.conn_stats_map.len() == 0 {
            return;
        }

        println!("========== Connection Stats ==========");
        for (id, conn_stats) in &self.conn_stats_map {
            let conn_stats = conn_stats.lock().unwrap().clone_and_clear_stats();
            let read_bps = conn_stats.bytes_read * 8 * 1000 / (self.config.report_interval.as_millis() as usize);
            let write_bps = conn_stats.bytes_write * 8 * 1000 / (self.config.report_interval.as_millis() as usize);
            println!(
                "[{}] {} => Read = {} bytes ({} bps), Write = {} bytes ({} bps)",
                id, conn_stats.remote_address, read_bps, conn_stats.bytes_read, write_bps, conn_stats.bytes_write
            );
        }
        println!();

        // We clean up the dead connections after reporting, otherwise we will miss the stats in the last round of report.
        self.conn_stats_map.retain(|_, v| v.lock().unwrap().is_alive);
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
    #[tracing::instrument(name = "Creating new TCP connection worker", level = "debug", skip(stream, conn_stats))]
    fn new(
        id: u32,
        config: Arc<RnpStubServerConfig>,
        stream: TcpStream,
        remote_address: SocketAddr,
        conn_stats: Arc<Mutex<StubServerTcpConnectionStats>>,
    ) -> StubServerTcpConnection {
        return StubServerTcpConnection { id, config, stream, remote_address, read_buf: vec![0; 4096], conn_stats };
    }

    #[tracing::instrument(name = "Running new TCP connection worker", level = "debug", skip(self), fields(id = %self.id, remote_address = %self.remote_address))]
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let result = self.run_loop().await;
        self.conn_stats.lock().unwrap().is_alive = false;
        return result;
    }

    async fn run_loop(&mut self) -> Result<(), Box<dyn Error>> {
        let mut interest = Interest::READABLE;
        if self.config.write_chunk_size != 0 {
            interest = interest | Interest::WRITABLE;
        }
        tracing::debug!(
            "Running TCP connection worker loop with interest, worker_id = {}, remote_address = {}, interest = {:?}",
            self.id,
            self.remote_address,
            interest
        );

        loop {
            let ready = self.stream.ready(interest).await?;

            if ready.is_readable() {
                self.on_connection_read().await?;
            }

            if ready.is_writable() {
                self.on_connection_write().await?;
            }
        }
    }

    #[tracing::instrument(name = "TCP connection on read", level = "debug", skip(self), fields(id = %self.id, remote_address = %self.remote_address))]
    async fn on_connection_read(&mut self) -> Result<(), Box<dyn Error>> {
        match self.stream.try_read(&mut self.read_buf) {
            Ok(n) => {
                if n == 0 {
                    if !self.config.wait_before_disconnect.is_zero() {
                        println!(
                            "Connection is half shutdown by remote side. Wait for {:?} before disconnect the connection: Remote = {}",
                            self.config.wait_before_disconnect, self.remote_address
                        );
                        tokio::time::sleep(self.config.wait_before_disconnect).await;
                    }

                    let error_message = format!("Connection is half shutdown by remote side. Closing connection: Remote = {}", self.remote_address);
                    println!("{}", error_message);

                    return Err(io::Error::new(io::ErrorKind::ConnectionAborted, error_message).into());
                }
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

    #[tracing::instrument(name = "TCP connection on write", level = "debug", skip(self), fields(id = %self.id, remote_address = %self.remote_address))]
    async fn on_connection_write(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.config.sleep_before_write.is_zero() {
            tokio::time::sleep(self.config.sleep_before_write).await;
        }

        // Update write count
        {
            let mut conn_stats = self.conn_stats.lock().unwrap();
            if self.config.write_count_limit != 0 && conn_stats.total_write_count >= self.config.write_count_limit {
                return Ok(());
            };
            conn_stats.total_write_count += 1;
        }

        // Write buffer
        let write_buf = vec![0 as u8; self.config.write_chunk_size];
        match self.stream.try_write(&write_buf) {
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
    pub remote_address: SocketAddr,
    pub is_alive: bool,
    pub bytes_read: usize,
    pub bytes_write: usize,
    pub total_write_count: u32,
}

impl StubServerTcpConnectionStats {
    pub fn new(remote_address: &SocketAddr) -> StubServerTcpConnectionStats {
        return StubServerTcpConnectionStats {
            remote_address: remote_address.clone(),
            is_alive: true,
            bytes_read: 0,
            bytes_write: 0,
            total_write_count: 0,
        };
    }

    pub fn clone_and_clear_stats(&mut self) -> StubServerTcpConnectionStats {
        let stats = self.clone();
        self.clear_stats();
        return stats;
    }

    pub fn clear_stats(&mut self) {
        self.bytes_write = 0;
        self.bytes_read = 0;
    }
}
