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
            return Err(PingClientPingResultDetails::new(rtt, Some(e)));
        }

        return Ok(PingClientPingResultDetails::new(rtt, None));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::net::SocketAddr;

    #[test]
    fn ping_client_tcp_should_fail_when_pinging_non_existing_host()
    {
        let source = SockAddr::from("0.0.0.0:0".parse::<SocketAddr>().unwrap());
        let target = SockAddr::from("1.1.1.1:11111".parse::<SocketAddr>().unwrap());

        let config = PingClientConfig { wait_timeout: Duration::from_millis(300), time_to_live: None };
        let ping_client = PingClientTcp::new(&config);
        let result = ping_client.ping(&source, &target);

        assert!(result.is_err());
        assert!(result.as_ref().err().is_some());
        assert!(result.as_ref().err().as_ref().unwrap().inner_error.is_some());
        assert_eq!(io::ErrorKind::TimedOut, result.as_ref().err().as_ref().unwrap().inner_error.as_ref().unwrap().kind());

        assert!(result.as_ref().err().as_ref().unwrap().round_trip_time.as_millis() > 200);
    }

    #[test]
    fn ping_client_tcp_should_fail_when_pinging_non_existing_port()
    {
        let source = SockAddr::from("0.0.0.0:0".parse::<SocketAddr>().unwrap());
        let target = SockAddr::from("127.0.0.1:56789".parse::<SocketAddr>().unwrap());

        let config = PingClientConfig { wait_timeout: Duration::from_millis(300), time_to_live: None };
        let ping_client = PingClientTcp::new(&config);
        let result = ping_client.ping(&source, &target);

        assert!(result.is_err());
        assert!(result.as_ref().err().is_some());
        assert!(result.as_ref().err().as_ref().unwrap().inner_error.is_some());

        // When connecting to a non existing port, on windows, it will timeout, but on other *nix OS, it will reject the connection.
        if cfg!(windows) {
            assert_eq!(io::ErrorKind::TimedOut, result.as_ref().err().as_ref().unwrap().inner_error.as_ref().unwrap().kind());
            assert!(result.as_ref().err().as_ref().unwrap().round_trip_time.as_millis() > 200);
        } else {
            assert_eq!(io::ErrorKind::ConnectionRefused, result.as_ref().err().as_ref().unwrap().inner_error.as_ref().unwrap().kind());
            assert_eq!(0, result.as_ref().err().as_ref().unwrap().round_trip_time.as_millis())
        }
    }

    #[test]
    fn ping_client_tcp_should_fail_when_binding_invalid_source_ip()
    {
        let source = SockAddr::from("1.1.1.1:1111".parse::<SocketAddr>().unwrap());
        let target = SockAddr::from("127.0.0.1:56789".parse::<SocketAddr>().unwrap());

        let config = PingClientConfig { wait_timeout: Duration::from_millis(300), time_to_live: None };
        let ping_client = PingClientTcp::new(&config);
        let result = ping_client.ping(&source, &target);

        assert!(result.is_err());
        assert!(result.as_ref().err().is_some());
        assert!(result.as_ref().err().as_ref().unwrap().inner_error.is_some());
        assert_eq!(io::ErrorKind::AddrNotAvailable, result.as_ref().err().as_ref().unwrap().inner_error.as_ref().unwrap().kind());
    }
}
