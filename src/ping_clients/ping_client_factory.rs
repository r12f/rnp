use crate::{PingClient, PingClientConfig, RnpSupportedProtocol};
use crate::ping_clients::ping_client_tcp::PingClientTcp;

#[cfg(any(not(target_os = "windows"), not(target_arch = "aarch64")))]
use crate::ping_clients::ping_client_quic::PingClientQuic;

pub fn new(
    protocol: RnpSupportedProtocol,
    config: &PingClientConfig,
) -> Box<dyn PingClient + Send + Sync> {
    match protocol {
        RnpSupportedProtocol::TCP => return Box::new(PingClientTcp::new(config)),
        RnpSupportedProtocol::QUIC => {
            if cfg!(any(not(target_os = "windows"), not(target_arch = "aarch64"))) {
                return Box::new(PingClientQuic::new(config));
            } else {
                panic!("Sorry, QUIC is not supported yet for Windows ARM64.");
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::ping_clients::ping_client_factory::new;
    use crate::{PingClientConfig, RnpSupportedProtocol};
    use std::time::Duration;

    #[test]
    fn create_tcp_ping_client_should_work() {
        let config = PingClientConfig {
            wait_timeout: Duration::from_millis(100),
            time_to_live: Some(128),
            use_fin_in_tcp_ping: false,
            server_name: None,
            log_tls_key: false,
            alpn_protocol: None
        };

        let ping_client = new(RnpSupportedProtocol::TCP, &config);
        assert_eq!("TCP", ping_client.protocol());
    }
}
