use crate::ping_clients::ping_client::{
    PingClient, PingClientPingResult, PingClientPingResultDetails,
};
use crate::PingClientConfig;
use contracts::{ensures, requires};
use quinn::crypto::rustls::TlsSession;
use quinn::{ClientConfig, ClientConfigBuilder, Endpoint, EndpointBuilder};
use rustls::ServerCertVerified;
use socket2::{Domain, SockAddr, Socket, Type};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct PingClientQuic {
    config: PingClientConfig,
    endpoint_builder: EndpointBuilder,
    endpoint: Option<Endpoint>,
}

impl PingClientQuic {
    pub fn new(config: &PingClientConfig) -> PingClientQuic {
        let mut client_config = ClientConfigBuilder::default().build();
        {
            let tls_cfg: &mut rustls::ClientConfig =
                Arc::get_mut(&mut client_config.crypto).unwrap();
            tls_cfg
                .dangerous()
                .set_certificate_verifier(Arc::new(SkipCertificationVerification));
        }

        let mut endpoint_builder = Endpoint::builder();
        endpoint_builder.default_client_config(client_config);

        return PingClientQuic {
            config: config.clone(),
            endpoint_builder,
            endpoint: None,
        };
    }

    fn create_local_endpoint(&mut self, source: &SockAddr) -> io::Result<()> {
        let source_addr = source.as_socket()?;
        let (endpoint, _) = self.endpoint_builder.bind(&source_addr)?;
        self.endpoint = Some(endpoint);
        return Ok(());
    }

    async fn ping_target(&self, target: &SockAddr) -> PingClientPingResult {
        let target_addr = target.as_socket().unwrap();
        let server_name = target_addr.to_string();

        let start_time = Instant::now();
        let connecting = self.endpoint.as_ref().unwrap().connect(&target_addr, &server_name)?;
        let connection = connecting.await?;
        let _rtt = Instant::now().duration_since(start_time);

        drop(connection);
        self.endpoint.wait_idle().await;
    }
}

impl PingClient for PingClientQuic {
    fn protocol(&self) -> &'static str {
        "QUIC"
    }

    #[ensures(ret.is_ok() -> self.socket.is_some())]
    #[ensures(ret.is_err() -> self.socket.is_none())]
    fn prepare_for_ping(&mut self, source: &SockAddr) -> io::Result<()> {
        return self.create_local_endpoint(source);
    }

    #[requires(self.socket.is_some())]
    fn ping(&self, target: &SockAddr) -> PingClientPingResult {
        return self.ping_target(target);
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
