use crate::*;
use async_trait::async_trait;
use quinn::{ClientConfig, ConnectionError, Endpoint, EndpointConfig, TransportConfig};
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::{Certificate, Error, ServerName};
use std::convert::TryInto;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

pub struct PingClientQuic {
    config: PingClientConfig,
}

impl PingClientQuic {
    pub fn new(config: &PingClientConfig) -> PingClientQuic {
        return PingClientQuic { config: config.clone() };
    }

    #[tracing::instrument(name = "Running QUIC ping in ping client", level = "debug", skip(self))]
    async fn ping_target(&self, source: &SocketAddr, target: &SocketAddr) -> PingClientResult<PingClientPingResultDetails> {
        let endpoint = self.create_local_endpoint(source).map_err(|e| PingClientError::PreparationFailed(Box::new(e)))?;
        let server_name = self.config.server_name.as_ref().map_or("", |s| &s);
        let use_timer_rtt = self.config.use_timer_rtt;
        let ping_result = PingClientQuic::connect_to_target(&endpoint, source, target, server_name, use_timer_rtt).await;
        endpoint.wait_idle().await;
        return ping_result;
    }

    fn create_local_endpoint(&self, source: &SocketAddr) -> io::Result<Endpoint> {
        let client_config = self.create_client_config();

        let socket = std::net::UdpSocket::bind(source)?;
        if let Some(ttl) = self.config.time_to_live {
            socket.set_ttl(ttl)?;
        }

        let endpoint_config = EndpointConfig::default();
        let runtime = quinn::TokioRuntime {};
        let mut endpoint = Endpoint::new(endpoint_config, None, socket, runtime)?;
        endpoint.set_default_client_config(client_config);
        return Ok(endpoint);
    }

    fn create_client_config(&self) -> ClientConfig {
        let roots = rustls::RootCertStore::empty();

        let mut client_crypto = rustls::ClientConfig::builder().with_safe_defaults().with_root_certificates(roots).with_no_client_auth();

        // Setup ALPN protocol if specified.
        if let Some(alpn_protocol) = &self.config.alpn_protocol {
            let protocols = vec![alpn_protocol.as_bytes().to_vec()];
            client_crypto.alpn_protocols = protocols;
        } else {
            client_crypto.alpn_protocols = vec![b"h3".to_vec()];
        }

        // Key logger
        if self.config.log_tls_key {
            client_crypto.key_log = Arc::new(rustls::KeyLogFile::new());
        }

        // Our main goal is to check network reachability, so we are not validating cert here.
        client_crypto.dangerous().set_certificate_verifier(Arc::new(SkipCertificationVerification));

        let mut transport_config = TransportConfig::default();
        transport_config.max_idle_timeout(Some(self.config.wait_timeout.try_into().unwrap()));

        let mut client_config = quinn::ClientConfig::new(Arc::new(client_crypto));
        client_config.transport_config(Arc::new(transport_config));

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

        let connecting = endpoint.connect(target.clone(), &server_name).map_err(|e| PingClientError::PingFailed(Box::new(e)))?;
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
                }
                ConnectionError::LocallyClosed => Err(PingClientError::PingFailed(Box::new(e))),
                _ => {
                    return Ok(PingClientPingResultDetails::new(None, rtt, false, Some(PingClientWarning::AppHandshakeFailed(Box::new(e)))));
                }
            },
        }?;

        let local_ip = connection.local_ip().map_or(None, |addr| Some(SocketAddr::new(addr, source.port())));
        if !use_timer_rtt {
            rtt = connection.rtt();
        }
        return Ok(PingClientPingResultDetails::new(local_ip, rtt, false, None));
    }
}

#[async_trait]
impl PingClient for PingClientQuic {
    fn protocol(&self) -> &'static str {
        "QUIC"
    }

    async fn prepare_ping(&mut self, _: &SocketAddr) -> Result<(), PingClientError> {
        Ok(())
    }

    async fn ping(&self, source: &SocketAddr, target: &SocketAddr) -> PingClientResult<PingClientPingResultDetails> {
        return self.ping_target(source, target).await;
    }
}

struct SkipCertificationVerification;

impl ServerCertVerifier for SkipCertificationVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}
