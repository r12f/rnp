use crate::ping_clients::ping_client::{PingClient, PingClientResult, PingClientPingResultDetails, PingClientError};
use crate::PingClientConfig;
use socket2::{Domain, SockAddr, Socket, Type};
use std::io;
use std::time::{Duration, Instant};

pub struct PingClientTcp {
    config: PingClientConfig,
}

impl PingClientTcp {
    pub fn new(config: &PingClientConfig) -> PingClientTcp {
        return PingClientTcp {
            config: config.clone(),
        };
    }

    fn ping_target(&self, source: &SockAddr, target: &SockAddr) -> PingClientResult<PingClientPingResultDetails> {
        let socket = self.prepare_socket_for_ping(source).map_err(|e| PingClientError::PreparationFailed(Box::new(e)))?;

        let start_time = Instant::now();
        let connect_result = socket.connect_timeout(target, self.config.wait_timeout);
        let rtt = Instant::now().duration_since(start_time);
        match connect_result {
            // Timeout is an expected value instead of an actual failure, so here we should return Ok.
            Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                return Ok(PingClientPingResultDetails::new(None, rtt, true))
            }
            Err(e) => return Err(PingClientError::PingFailed(Box::new(e))),
            Ok(()) => (),
        }

        // If getting local address failed, we ignore it.
        // The worse case we can get is to output a 0.0.0.0 as source IP, which is not critical to what we are trying to do.
        let local_addr = socket.local_addr();
        return match local_addr {
            Ok(addr) => Ok(PingClientPingResultDetails::new(Some(addr), rtt, false)),
            Err(_) => Ok(PingClientPingResultDetails::new(None, rtt, false)),
        };
    }

    fn prepare_socket_for_ping(&self, source: &SockAddr) -> io::Result<Socket> {
        let socket_domain = Domain::from(source.family() as i32);
        let socket = Socket::new(socket_domain, Type::STREAM, None)?;
        socket.bind(&source)?;
        if !self.config.use_fin_in_tcp_ping {
            socket.set_linger(Some(Duration::from_secs(0)))?;
        }
        if let Some(ttl) = self.config.time_to_live {
            socket.set_ttl(ttl)?;
        }

        return Ok(socket);
    }
}

impl PingClient for PingClientTcp {
    fn protocol(&self) -> &'static str {
        "TCP"
    }

    fn ping(&self, source: &SockAddr, target: &SockAddr) -> PingClientResult<PingClientPingResultDetails> {
        return self.ping_target(source, target);
    }
}
