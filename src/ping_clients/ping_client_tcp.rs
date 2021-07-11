use crate::ping_clients::ping_client::{PingClient, PingClientPingResult, PingClientPingResultDetails};
use crate::PingClientConfig;
use socket2::{Domain, SockAddr, Socket, Type, Protocol};
use std::time::{Duration, Instant};

pub struct PingClientTcp {
    config: PingClientConfig,
}

impl PingClientTcp {
    pub fn new(config: &PingClientConfig) -> PingClientTcp {
        return PingClientTcp { config: config.clone() };
    }
}

impl PingClient for PingClientTcp {
    fn protocol(&self) -> Protocol { Protocol::TCP }

    fn ping(&self, source: &SockAddr, target: &SockAddr) -> PingClientPingResult {
        let socket_domain = Domain::from(target.family() as i32);
        let socket = Socket::new(socket_domain, Type::STREAM, None)?;
        socket.bind(&source)?;
        socket.set_reuse_address(true)?;
        socket.set_linger(Some(Duration::from_secs(0)))?;
        if let Some(ttl) = self.config.time_to_live {
            socket.set_ttl(ttl)?;
        }

        let start_time = Instant::now();
        let connect_result = socket.connect_timeout(&target, self.config.wait_timeout);
        let rtt = Instant::now().duration_since(start_time);
        if let Err(e) = connect_result {
            return Err(PingClientPingResultDetails::new(None, rtt, Some(e)));
        }

        let local_addr = socket.local_addr();
        return match local_addr {
            Ok(addr) => Ok(PingClientPingResultDetails::new(Some(addr), rtt, None)),
            Err(_) => Ok(PingClientPingResultDetails::new(None, rtt, None)),
        };
    }
}