use crate::*;
use async_trait::async_trait;
use quinn::{ClientConfig, ClientConfigBuilder, ConnectionError, Endpoint, EndpointError};
use rustls::ServerCertVerified;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

pub struct PingClientQuic {
    config: PingClientConfig,
}

impl PingClientQuic {
    pub fn new(config: &PingClientConfig) -> PingClientQuic {
        return PingClientQuic {
            config: config.clone(),
        };
    }

    #[tracing::instrument(name = "Running QUIC ping in ping client", level = "debug", skip(self))]
    async fn ping_target(
        &self,
        source: &SocketAddr,
        target: &SocketAddr,
    ) -> PingClientResult<PingClientPingResultDetails> {
        let endpoint = self
            .create_local_endpoint(source)
            .map_err(|e| PingClientError::PreparationFailed(Box::new(e)))?;
        let server_name = self.config.server_name.as_ref().map_or("", |s| &s);
        let use_timer_rtt = self.config.use_timer_rtt;
        let ping_result = PingClientQuic::connect_to_target(
            &endpoint,
            source,
            target,
            server_name,
            use_timer_rtt,
        )
        .await;
        endpoint.wait_idle().await;
        return ping_result;
    }

    fn create_local_endpoint(&self, source: &SocketAddr) -> Result<Endpoint, EndpointError> {
        let client_config = self.create_client_config();

        let mut endpoint_builder = Endpoint::builder();
        endpoint_builder.default_client_config(client_config);

        let socket = std::net::UdpSocket::bind(source).map_err(EndpointError::Socket)?;
        if let Some(ttl) = self.config.time_to_live {
            socket.set_ttl(ttl).map_err(EndpointError::Socket)?;
        }

        let (endpoint, _) = endpoint_builder.with_socket(socket)?;
        return Ok(endpoint);
    }

    fn create_client_config(&self) -> ClientConfig {
        let mut client_config_builder = ClientConfigBuilder::default();

        // Setup ALPN protocol if specified.
        if let Some(alpn_protocol) = &self.config.alpn_protocol {
            let protocols: &[&[u8]] = &[alpn_protocol.as_bytes()];
            client_config_builder.protocols(protocols);
        }

        if self.config.log_tls_key {
            client_config_builder.enable_keylog();
        }

        let mut client_config = client_config_builder.build();
        {
            let tls_cfg: &mut rustls::ClientConfig = Arc::get_mut(&mut client_config.crypto)
                .expect("Failed to get QUIC client crypto config, which should never happen.");

            tls_cfg
                .dangerous()
                .set_certificate_verifier(Arc::new(SkipCertificationVerification));

            let transport_config = Arc::get_mut(&mut client_config.transport)
                .expect("Failed to get QUIC client transport config, which should never happen.");

            transport_config
                .max_idle_timeout(Some(self.config.wait_timeout))
                .expect("Failed to set QUIC client max idle timeout, which should never happen.");
        }

        return client_config;
    }

    async fn connect_to_target(
        endpoint: &Endpoint,
        source: &SocketAddr,
        target: &SocketAddr,
        server_name: &str,
        use_timer_rtt: bool,
    ) -> PingClientResult<PingClientPingResultDetails> {
        let start_time = Instant::now();

        let connecting = endpoint
            .connect(target, &server_name)
            .map_err(|e| PingClientError::PingFailed(Box::new(e)))?;
        let connecting_result = connecting.await;
        let mut rtt = Instant::now().duration_since(start_time);

        // If a QUIC connection returned errors other than timed out or local error, it means the local endpoint has successfully
        // received packets from remote server, which means the underlying network is reachable, but higher level of stack went
        // wrong, such as ALPN, so here, we should log this failure as warning instead.
        let connection = match connecting_result {
            Ok(connection) => Ok(connection),
            Err(e) => match e {
                ConnectionError::TimedOut => {
                    return Ok(PingClientPingResultDetails::new(None, rtt, true, None));
                },
                ConnectionError::LocallyClosed => Err(PingClientError::PingFailed(Box::new(e))),
                _ => {
                    return Ok(PingClientPingResultDetails::new(
                        None,
                        rtt,
                        false,
                        Some(PingClientWarning::AppHandshakeFailed(Box::new(e))),
                    ));
                }
            },
        }?;

        let local_ip = connection
            .connection
            .local_ip()
            .map_or(None, |addr| Some(SocketAddr::new(addr, source.port())));
        if !use_timer_rtt {
            rtt = connection.connection.rtt();
        }
        return Ok(PingClientPingResultDetails::new(local_ip, rtt, false, None));
    }
}

#[async_trait]
impl PingClient for PingClientQuic {
    fn protocol(&self) -> &'static str {
        "QUIC"
    }

    async fn ping(
        &self,
        source: &SocketAddr,
        target: &SocketAddr,
    ) -> PingClientResult<PingClientPingResultDetails> {
        return self.ping_target(source, target).await;
    }
}

struct SkipCertificationVerification;

impl rustls::ServerCertVerifier for SkipCertificationVerification {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}

#[cfg(test)]
mod tests {
    use crate::ping_clients::ping_client_test_common::*;
    use crate::{ping_clients::ping_client_factory, PingClientConfig, RnpSupportedProtocol};
    use futures_intrusive::sync::ManualResetEvent;
    use std::sync::Arc;
    use std::time::Duration;
    use tide::prelude::*;
    use tide::Request;
    use tokio::runtime::Runtime;

    #[test]
    fn ping_client_quic_should_work() {
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
                check_disconnect: false,
                server_name: Some("localhost".to_string()),
                log_tls_key: false,
                alpn_protocol: Some("hq-29".to_string()),
                use_timer_rtt: false,
            };
            let mut ping_client = ping_client_factory::new(&RnpSupportedProtocol::QUIC, &config, None);

            // When connecting to a non existing port, on windows, it will timeout, but on other *nix OS, it will reject the connection.
            let expected_results = ExpectedPingClientTestResults {
                timeout_min_time: Duration::from_millis(200),
                ping_non_existing_host_result: ExpectedTestCaseResult::Timeout,
                ping_non_existing_port_result: if cfg!(windows) {
                    ExpectedTestCaseResult::Timeout
                } else {
                    ExpectedTestCaseResult::Failed("connection refused")
                },
                binding_invalid_source_ip_result: ExpectedTestCaseResult::Failed("failed to set up UDP socket: The requested address is not valid in its context. (os error 10049)"),
                binding_unavailable_source_port_result: ExpectedTestCaseResult::Failed("failed to set up UDP socket: Only one usage of each socket address (protocol/network address/port) is normally permitted. (os error 10048)"),
            };

            run_ping_client_tests(&mut ping_client, &"127.0.0.1:4433".parse().unwrap(), &expected_results).await;
        });
    }

    async fn valid_http_handler(_req: Request<()>) -> tide::Result {
        Ok("It works!".into())
    }
}
