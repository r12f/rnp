use crate::ping_clients::ping_client::{PingClient, PingClientResult, PingClientPingResultDetails, PingClientError};
use crate::PingClientConfig;
use socket2::{Domain, SockAddr, Socket, Type};
use std::io;
use std::time::{Duration, Instant};
use std::net::SocketAddr;
use async_trait::async_trait;

pub struct PingClientTcp {
    config: PingClientConfig,
}

impl PingClientTcp {
    pub fn new(config: &PingClientConfig) -> PingClientTcp {
        return PingClientTcp {
            config: config.clone(),
        };
    }

    #[tracing::instrument(name = "Running TCP ping in ping client", level = "debug", skip(self))]
    fn ping_target(&self, source: &SocketAddr, target: &SocketAddr) -> PingClientResult<PingClientPingResultDetails> {
        let socket = self.prepare_socket_for_ping(source).map_err(|e| PingClientError::PreparationFailed(Box::new(e)))?;

        let start_time = Instant::now();
        let connect_result = socket.connect_timeout(&SockAddr::from(target.clone()), self.config.wait_timeout);
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
            Ok(addr) => Ok(PingClientPingResultDetails::new(Some(addr.as_socket().unwrap()), rtt, false)),
            Err(_) => Ok(PingClientPingResultDetails::new(None, rtt, false)),
        };
    }

    fn prepare_socket_for_ping(&self, source: &SocketAddr) -> io::Result<Socket> {
        let socket_domain = if source.is_ipv4() { Domain::IPV4 } else { Domain::IPV6 };
        let socket = Socket::new(socket_domain, Type::STREAM, None)?;
        socket.bind(&SockAddr::from(source.clone()))?;
        if !self.config.use_fin_in_tcp_ping {
            socket.set_linger(Some(Duration::from_secs(0)))?;
        }
        if let Some(ttl) = self.config.time_to_live {
            socket.set_ttl(ttl)?;
        }

        return Ok(socket);
    }
}

#[async_trait]
impl PingClient for PingClientTcp {
    fn protocol(&self) -> &'static str {
        "TCP"
    }

    async fn ping(&self, source: &SocketAddr, target: &SocketAddr) -> PingClientResult<PingClientPingResultDetails> {
        return self.ping_target(source, target);
    }
}

#[cfg(test)]
mod tests {
    use crate::{ping_clients::ping_client_factory, PingClientConfig, RnpSupportedProtocol};
    use futures_intrusive::sync::ManualResetEvent;
    use std::sync::Arc;
    use tide::prelude::*;
    use tide::Request;
    use tokio::runtime::Runtime;
    use std::time::Duration;
    use crate::ping_clients::ping_client_test_common::*;

    #[test]
    fn ping_client_tcp_should_work() {
        let rt = Runtime::new().unwrap();

        let ready_event = Arc::new(ManualResetEvent::new(false));
        let ready_event_clone = ready_event.clone();
        let _server = rt.spawn(async move {
            let mut app = tide::new();
            app.at("/test").get(valid_http_handler);
            let mut listener = app.bind("127.0.0.1:11337").await.unwrap();
            ready_event_clone.set();
            listener.accept().await.unwrap_or_default();
        });
        rt.block_on(ready_event.wait());

        rt.block_on(async {
            let config = PingClientConfig {
                wait_timeout: Duration::from_millis(300),
                time_to_live: None,
                use_fin_in_tcp_ping: false,
                server_name: None,
            };
            let mut ping_client = ping_client_factory::new(RnpSupportedProtocol::TCP, &config);

            // When connecting to a non existing port, on windows, it will timeout, but on other *nix OS, it will reject the connection.
            let expected_results = ExpectedPingClientTestResults {
                timeout_min_time: Duration::from_millis(200),
                ping_non_existing_host_result: ExpectedTestCaseResult::Timeout,
                ping_non_existing_port_result: if cfg!(windows) {
                    ExpectedTestCaseResult::Timeout
                } else {
                    ExpectedTestCaseResult::Failed("connection refused")
                },
                binding_invalid_source_ip_result: ExpectedTestCaseResult::Failed("The requested address is not valid in its context. (os error 10049)"),
                binding_unavailable_source_port_result: ExpectedTestCaseResult::Failed("Only one usage of each socket address (protocol/network address/port) is normally permitted. (os error 10048)"),
            };

            run_ping_client_tests(&mut ping_client, &expected_results).await;
        });
    }

    async fn valid_http_handler(_req: Request<()>) -> tide::Result { Ok("It works!".into()) }
}
