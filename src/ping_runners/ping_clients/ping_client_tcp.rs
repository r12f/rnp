use crate::*;
use async_trait::async_trait;
use socket2::{Domain, SockAddr, Socket, Type};
use std::io;
use std::mem::MaybeUninit;
use std::net::{Shutdown, SocketAddr};
use std::time::{Duration, Instant};

pub struct PingClientTcp {
    config: PingClientConfig,
}

impl PingClientTcp {
    pub fn new(config: &PingClientConfig) -> PingClientTcp {
        return PingClientTcp { config: config.clone() };
    }

    #[tracing::instrument(name = "Running TCP ping in ping client", level = "debug", skip(self))]
    async fn ping_target(&self, source: &SocketAddr, target: &SocketAddr) -> PingClientResult<PingClientPingResultDetails> {
        let socket = self.prepare_socket_for_ping(source).map_err(|e| PingClientError::PreparationFailed(Box::new(e)))?;

        let start_time = Instant::now();
        let connect_result = socket.connect_timeout(&SockAddr::from(target.clone()), self.config.wait_timeout);
        let rtt = Instant::now().duration_since(start_time);
        match connect_result {
            // Timeout is an expected value instead of an actual failure, so here we should return Ok.
            Err(e) if e.kind() == io::ErrorKind::TimedOut => return Ok(PingClientPingResultDetails::new(None, rtt, true, None)),
            Err(e) => return Err(PingClientError::PingFailed(Box::new(e))),
            Ok(()) => (),
        }
        let local_addr = socket.local_addr();

        // Check closing connection as well as opening connection
        let mut warning: Option<PingClientWarning> = None;
        if self.config.check_disconnect {
            warning = match self.shutdown_connection(socket, &target).await {
                Err(e) => Some(PingClientWarning::DisconnectFailed(Box::new(e))),
                Ok(_) => None,
            }
        }

        // If getting local address failed, we ignore it.
        // The worse case we can get is to output a 0.0.0.0 as source IP, which is not critical to what we are trying to do.
        return match local_addr {
            Ok(addr) => Ok(PingClientPingResultDetails::new(Some(addr.as_socket().unwrap()), rtt, false, warning)),
            Err(_) => Ok(PingClientPingResultDetails::new(None, rtt, false, warning)),
        };
    }

    #[tracing::instrument(name = "Creating socket for ping", level = "debug", skip(self))]
    fn prepare_socket_for_ping(&self, source: &SocketAddr) -> io::Result<Socket> {
        let socket_domain = if source.is_ipv4() { Domain::IPV4 } else { Domain::IPV6 };
        let socket = Socket::new(socket_domain, Type::STREAM, None)?;
        socket.bind(&SockAddr::from(source.clone()))?;
        socket.set_read_timeout(Some(self.config.wait_timeout))?;
        if !self.config.check_disconnect {
            socket.set_linger(Some(Duration::from_secs(0)))?;
        }
        if let Some(ttl) = self.config.time_to_live {
            socket.set_ttl(ttl)?;
        }

        return Ok(socket);
    }

    #[tracing::instrument(name = "Shutdown connection after ping", level = "debug", skip(self))]
    async fn shutdown_connection(&self, socket: Socket, target: &SocketAddr) -> io::Result<()> {
        if !self.config.wait_before_disconnect.is_zero() {
            tracing::debug!("Waiting {:?} before disconnect; target = {}", self.config.wait_before_disconnect, target);
            tokio::time::sleep(self.config.wait_before_disconnect).await;
        }

        tracing::debug!("Shutdown connection write; target = {}", target);
        socket.shutdown(Shutdown::Write)?;

        // Try to read until recv returns nothing, which indicates shutdown is succeeded.
        tracing::debug!("Wait until shutdown completes; target = {}", target);
        let mut buf: [MaybeUninit<u8>; 128] = unsafe { MaybeUninit::uninit().assume_init() };
        while socket.recv(&mut buf)? > 0 {
            continue;
        }

        return Ok(());
    }
}

#[async_trait]
impl PingClient for PingClientTcp {
    fn protocol(&self) -> &'static str {
        "TCP"
    }

    async fn prepare_ping(&mut self, _: &SocketAddr) -> Result<(), PingClientError> {
        Ok(())
    }

    async fn ping(&self, source: &SocketAddr, target: &SocketAddr) -> PingClientResult<PingClientPingResultDetails> {
        return self.ping_target(source, target).await;
    }
}
