use crate::ping_clients::ping_client_tcp::PingClientTcp;
use crate::*;

#[cfg(any(not(target_os = "windows"), not(target_arch = "aarch64")))]
use crate::ping_clients::ping_client_quic::PingClientQuic;

pub type PingClientFactory = fn(
    protocol: &RnpSupportedProtocol,
    config: &PingClientConfig,
) -> Option<Box<dyn PingClient + Send + Sync>>;

pub fn new_ping_client(
    protocol: &RnpSupportedProtocol,
    config: &PingClientConfig,
    external_ping_client_factory: Option<PingClientFactory>,
) -> Box<dyn PingClient + Send + Sync> {
    if let Some(factory) = external_ping_client_factory {
        if let Some(ping_client) = factory(protocol, config) {
            return ping_client;
        }
    }

    return new_inbox_ping_client(protocol, config);
}

#[cfg(any(not(target_os = "windows"), not(target_arch = "aarch64")))]
fn new_inbox_ping_client(
    protocol: &RnpSupportedProtocol,
    config: &PingClientConfig,
) -> Box<dyn PingClient + Send + Sync> {
    match protocol {
        RnpSupportedProtocol::TCP => return Box::new(PingClientTcp::new(config)),
        RnpSupportedProtocol::QUIC => return Box::new(PingClientQuic::new(config)),
        RnpSupportedProtocol::External(p) => panic!("Protocol {} is not supported!", p),
    }
}

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
fn new_inbox_ping_client(
    protocol: &RnpSupportedProtocol,
    config: &PingClientConfig,
) -> Box<dyn PingClient + Send + Sync> {
    match protocol {
        RnpSupportedProtocol::TCP => return Box::new(PingClientTcp::new(config)),
        RnpSupportedProtocol::QUIC => {
            panic!("Sorry, QUIC ping is not supported yet for Windows ARM64.")
        }
        RnpSupportedProtocol::External(p) => panic!(format!("Protocol {} is not supported!", p)),
    }
}

#[cfg(test)]
mod tests {
    use crate::ping_clients::ping_client_factory::new_ping_client;
    use crate::{PingClientConfig, RnpSupportedProtocol};
    use std::time::Duration;

    #[test]
    fn create_tcp_ping_client_should_work() {
        let config = PingClientConfig {
            wait_timeout: Duration::from_millis(100),
            time_to_live: Some(128),
            check_disconnect: false,
            server_name: None,
            log_tls_key: false,
            alpn_protocol: None,
            use_timer_rtt: false,
        };

        let ping_client = new_ping_client(&RnpSupportedProtocol::TCP, &config, None);
        assert_eq!("TCP", ping_client.protocol());
    }
}
