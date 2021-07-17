use crate::ping_clients::ping_client_tcp::PingClientTcp;
use crate::{PingClient, PingClientConfig, RnpSupportedProtocol};
use contracts::requires;

#[requires(protocol == RnpSupportedProtocol::TCP)]
pub fn new(protocol: RnpSupportedProtocol, config: &PingClientConfig) -> Box<dyn PingClient + Send + Sync> {
    match protocol {
        RnpSupportedProtocol::TCP => return Box::new(PingClientTcp::new(config)),
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
        };

        let ping_client = new(RnpSupportedProtocol::TCP, &config);
        assert_eq!("TCP", ping_client.protocol());
    }
}
