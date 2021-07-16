use crate::ping_clients::ping_client::{PingClient, PingClientPingResult, PingClientPingResultDetails};
use crate::PingClientConfig;
use socket2::{Domain, SockAddr, Socket, Type};
use std::time::{Duration, Instant};
use std::io;
use contracts::{requires, ensures};

pub struct PingClientTcp {
    config: PingClientConfig,
    socket: Option<Socket>,
}

impl PingClientTcp {
    pub fn new(config: &PingClientConfig) -> PingClientTcp {
        return PingClientTcp { config: config.clone(), socket: None };
    }

    fn prepare_tcp_socket(&mut self, source: &SockAddr) -> io::Result<()> {
        self.socket = None;

        let socket_domain = Domain::from(source.family() as i32);
        let socket = Socket::new(socket_domain, Type::STREAM, None)?;
        socket.bind(&source)?;
        if !self.config.use_fin_in_tcp_ping {
            socket.set_linger(Some(Duration::from_secs(0)))?;
        }
        if let Some(ttl) = self.config.time_to_live {
            socket.set_ttl(ttl)?;
        }
        self.socket = Some(socket);

        return Ok(());
    }

    fn ping_target(&self, target: &SockAddr) -> PingClientPingResult {
        let start_time = Instant::now();
        let connect_result = self.socket.as_ref().unwrap().connect_timeout(target, self.config.wait_timeout);
        let rtt = Instant::now().duration_since(start_time);
        match connect_result {
            Err(e) if e.kind() == io::ErrorKind::TimedOut => return Err(PingClientPingResultDetails::new(None, rtt, Some(e))),
            Err(e) => return Err(PingClientPingResultDetails::new(None, Duration::from_millis(0), Some(e))),
            Ok(()) => (),
        }

        // If getting local address failed, we ignore it.
        // The worse case we can get is to output a 0.0.0.0 as source IP, which is not critical to what we are trying to do.
        let local_addr = self.socket.as_ref().unwrap().local_addr();
        return match local_addr {
            Ok(addr) => Ok(PingClientPingResultDetails::new(Some(addr), rtt, None)),
            Err(_) => Ok(PingClientPingResultDetails::new(None, rtt, None)),
        };
    }
}

impl PingClient for PingClientTcp {
    fn protocol(&self) -> &'static str { "TCP" }

    #[ensures(ret.is_ok() -> self.socket.is_some())]
    #[ensures(ret.is_err() -> self.socket.is_none())]
    fn prepare_for_ping(&mut self, source: &SockAddr) -> io::Result<()> {
        return self.prepare_tcp_socket(source);
    }

    #[requires(self.socket.is_some())]
    fn ping(&self, target: &SockAddr) -> PingClientPingResult {
        return self.ping_target(target);
    }
}