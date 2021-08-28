use crate::ping_clients::ping_client_test_common::*;
use crate::{ping_clients::ping_client_factory, rnp_test_common, PingClientConfig, RnpSupportedProtocol};
use std::time::Duration;
use tokio::runtime::Runtime;

#[test]
fn ping_client_quic_should_fail_when_pinging_non_existing_host() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let config = create_ping_client_quic_default_config();
        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::QUIC, &config, None);
        ping_client_should_fail_when_pinging_non_existing_host(&mut ping_client, &ExpectedTestCaseResult::Timeout).await;
    });
}

#[test]
fn ping_client_quic_should_fail_when_pinging_non_existing_port() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let config = create_ping_client_quic_default_config();
        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::QUIC, &config, None);
        ping_client_should_fail_when_pinging_non_existing_port(&mut ping_client, &ExpectedTestCaseResult::Timeout).await;
    });
}

#[test]
fn ping_client_quic_should_fail_when_binding_invalid_source_ip() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let config = create_ping_client_quic_default_config();
        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::QUIC, &config, None);

        let expected_result =
            ExpectedTestCaseResult::Failed("failed to set up UDP socket: The requested address is not valid in its context. (os error 10049)");
        ping_client_should_fail_when_binding_invalid_source_ip(&mut ping_client, &expected_result).await;
    });
}

fn create_ping_client_quic_default_config() -> PingClientConfig {
    return PingClientConfig {
        wait_timeout: Duration::from_millis(300),
        time_to_live: None,
        check_disconnect: false,
        server_name: Some("localhost".to_string()),
        log_tls_key: false,
        alpn_protocol: Some("hq-29".to_string()),
        use_timer_rtt: false,
    };
}
