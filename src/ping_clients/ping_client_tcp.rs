use crate::ping_clients::ping_client::{PingClient, PingClientPingResult, PingClientPingResultDetails};
use crate::PingClientConfig;
use socket2::{Domain, SockAddr, Socket, Type, Protocol};
use std::time::{Duration, Instant};
use std::io;

pub struct PingClientTcp {
    config: PingClientConfig,
}

impl PingClientTcp {
    pub fn new(config: &PingClientConfig) -> PingClientTcp {
        return PingClientTcp { config: config.clone() };
    }

    fn ping_target(&self, source: &SockAddr, target: &SockAddr) -> PingClientPingResult {
        let socket_domain = Domain::from(target.family() as i32);
        let socket = match self.prepare_socket_for_ping(socket_domain, source) {
            Ok(socket) => socket,
            Err(e) => return Err(PingClientPingResultDetails::new(None, Duration::from_millis(0), Some(e), None)),
        };

        let start_time = Instant::now();
        let connect_result = socket.connect_timeout(&target, self.config.wait_timeout);
        let rtt = Instant::now().duration_since(start_time);
        match connect_result {
            Err(e) if e.kind() == io::ErrorKind::TimedOut => return Err(PingClientPingResultDetails::new(None, rtt, None, Some(e))),
            Err(e) => return Err(PingClientPingResultDetails::new(None, Duration::from_millis(0), None, Some(e))),
            Ok(()) => (),
        }

        // If getting local address failed, we ignore it.
        // The worse case we can get is to output a 0.0.0.0 as source IP, which is not critical to what we are trying to do.
        let local_addr = socket.local_addr();
        return match local_addr {
            Ok(addr) => Ok(PingClientPingResultDetails::new(Some(addr), rtt, None, None)),
            Err(_) => Ok(PingClientPingResultDetails::new(None, rtt, None, None)),
        };
    }

    fn prepare_socket_for_ping(&self, socket_domain: Domain, source: &SockAddr) -> io::Result<Socket> {
        let socket = Socket::new(socket_domain, Type::STREAM, None)?;
        socket.bind(&source)?;
        socket.set_linger(Some(Duration::from_secs(0)))?;
        if let Some(ttl) = self.config.time_to_live {
            socket.set_ttl(ttl)?;
        }

        return Ok(socket);
    }
}

impl PingClient for PingClientTcp {
    fn protocol(&self) -> Protocol { Protocol::TCP }

    fn ping(&self, source: &SockAddr, target: &SockAddr) -> PingClientPingResult {
        return self.ping_target(source, target);
    }
}