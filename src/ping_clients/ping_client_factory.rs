use crate::ping_clients::ping_client_tcp::PingClientTcp;
use crate::{PingClient, PingClientConfig};
use contracts::requires;
use socket2::Protocol;

#[requires(protocol == Protocol::TCP)]
pub fn new(protocol: Protocol, config: &PingClientConfig) -> Box<dyn PingClient + Send + Sync> {
    match protocol {
        Protocol::TCP => return Box::new(PingClientTcp::new(config)),
        _ => panic!("Unexpected protocol type!"),
    }
}

#[cfg(test)]
mod tests {
    use crate::ping_clients::ping_client_factory::new;
    use crate::PingClientConfig;
    use socket2::Protocol;
    use std::time::Duration;

    #[test]
    fn create_tcp_ping_client_should_work() {
        let config = PingClientConfig {
            wait_timeout: Duration::from_millis(100),
            time_to_live: Some(128),
        };

        let ping_client = new(Protocol::TCP, &config);
        assert_eq!(Protocol::TCP, ping_client.protocol());
    }

    #[test]
    #[should_panic]
    fn create_udp_ping_client_should_panic() {
        let config = PingClientConfig {
            wait_timeout: Duration::from_millis(100),
            time_to_live: Some(128),
        };

        new(Protocol::UDP, &config);
    }
}
