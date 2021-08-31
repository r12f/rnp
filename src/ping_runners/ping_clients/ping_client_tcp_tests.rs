use crate::ping_clients::ping_client_test_common::*;
use crate::stub_servers::stub_server_factory;
use crate::{ping_clients::ping_client_factory, rnp_test_common, PingClientConfig, RnpStubServerConfig, RnpSupportedProtocol};
use futures_intrusive::sync::ManualResetEvent;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

#[test]
fn ping_client_tcp_should_work_when_pinging_good_host() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    let server_address = "127.0.0.1:11337".parse::<SocketAddr>().unwrap();
    let server_config = create_tcp_stub_server_default_config(&server_address);
    start_run_tcp_stub_server(&rt, server_config);

    rt.block_on(async move {
        let config = create_ping_client_tcp_default_config();
        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::TCP, &config, None);
        ping_client_should_work_when_pinging_good_host(&mut ping_client, &server_address).await;
    });
}

#[test]
fn ping_client_tcp_should_fail_when_binding_unavailable_source_port() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    let server_address = "127.0.0.1:11338".parse::<SocketAddr>().unwrap();
    let server_config = create_tcp_stub_server_default_config(&server_address);
    start_run_tcp_stub_server(&rt, server_config);

    rt.block_on(async move {
        let config = create_ping_client_tcp_default_config();
        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::TCP, &config, None);

        let expected_result = ExpectedTestCaseResult::Failed(
            "Only one usage of each socket address (protocol/network address/port) is normally permitted. (os error 10048)",
        );
        ping_client_should_fail_when_binding_unavailable_source_port(&mut ping_client, &server_address, &expected_result).await;
    });
}

#[test]
fn ping_client_tcp_should_work_when_pinging_good_host_with_check_disconnect() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    let server_address = "127.0.0.1:11339".parse::<SocketAddr>().unwrap();
    let server_config = create_tcp_stub_server_default_config(&server_address);
    start_run_tcp_stub_server(&rt, server_config);

    rt.block_on(async move {
        let mut config = create_ping_client_tcp_default_config();
        config.check_disconnect = true;

        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::TCP, &config, None);
        ping_client_should_work_when_pinging_good_host(&mut ping_client, &server_address).await;
    });
}

// #[test]
// fn ping_client_tcp_should_warn_when_server_closes_connection_before_disconnect() {
//     rnp_test_common::initialize();
//     let rt = Runtime::new().unwrap();
//
//     let server_address = "127.0.0.1:11340".parse::<SocketAddr>().unwrap();
//     let mut server_config = create_tcp_stub_server_default_config(&server_address);
//     server_config.close_on_accept = true;
//     start_run_tcp_stub_server(&rt, server_config);
//
//     rt.block_on(async move {
//         let mut config = create_ping_client_tcp_default_config();
//         config.check_disconnect = true;
//         config.wait_before_disconnect = Duration::from_millis(5000);
//
//         let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::TCP, &config, None);
//         let source = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
//         let expected_result = ExpectedTestCaseResult::Warning("Connection is already half shutdown by remote side.");
//         ping_client_result_should_be_expected(&mut ping_client, &source, &server_address, Duration::from_millis(200), &expected_result).await;
//     });
// }

#[test]
fn ping_client_tcp_should_warn_when_disconnect_timed_out() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    let server_address = "127.0.0.1:11340".parse::<SocketAddr>().unwrap();
    let mut server_config = create_tcp_stub_server_default_config(&server_address);
    server_config.wait_before_disconnect = Duration::from_millis(2000);
    start_run_tcp_stub_server(&rt, server_config);

    rt.block_on(async move {
        let mut config = create_ping_client_tcp_default_config();
        config.check_disconnect = true;
        config.disconnect_timeout = Duration::from_millis(100);

        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::TCP, &config, None);
        let source = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
        let expected_result = ExpectedTestCaseResult::Warning("Disconnect timed out.");
        ping_client_result_should_be_expected(&mut ping_client, &source, &server_address, Duration::from_millis(200), &expected_result).await;
    });
}

#[test]
fn ping_client_tcp_should_fail_when_pinging_non_existing_host() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let config = create_ping_client_tcp_default_config();
        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::TCP, &config, None);
        ping_client_should_fail_when_pinging_non_existing_host(&mut ping_client, &ExpectedTestCaseResult::Timeout).await;
    });
}

#[test]
fn ping_client_tcp_should_fail_when_pinging_non_existing_port() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let config = create_ping_client_tcp_default_config();
        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::TCP, &config, None);

        let expected_result = if cfg!(windows) { ExpectedTestCaseResult::Timeout } else { ExpectedTestCaseResult::Failed("connection refused") };
        ping_client_should_fail_when_pinging_non_existing_port(&mut ping_client, &expected_result).await;
    });
}

#[test]
fn ping_client_tcp_should_fail_when_binding_invalid_source_ip() {
    rnp_test_common::initialize();
    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let config = create_ping_client_tcp_default_config();
        let mut ping_client = ping_client_factory::new_ping_client(&RnpSupportedProtocol::TCP, &config, None);

        let expected_result = ExpectedTestCaseResult::Failed("The requested address is not valid in its context. (os error 10049)");
        ping_client_should_fail_when_binding_invalid_source_ip(&mut ping_client, &expected_result).await;
    });
}

fn create_tcp_stub_server_default_config(server_address: &SocketAddr) -> RnpStubServerConfig {
    return RnpStubServerConfig {
        protocol: RnpSupportedProtocol::TCP,
        server_address: server_address.clone(),
        close_on_accept: false,
        sleep_before_write: Duration::ZERO,
        write_chunk_size: 0,
        write_count_limit: 0,
        report_interval: Duration::from_secs(1),
        wait_before_disconnect: Duration::ZERO,
    };
}

fn start_run_tcp_stub_server(rt: &Runtime, stub_server_config: RnpStubServerConfig) {
    let ready_event = Arc::new(ManualResetEvent::new(false));
    let ready_event_clone = ready_event.clone();
    rt.spawn(async move {
        let _ = stub_server_factory::run(&stub_server_config, Arc::new(ManualResetEvent::new(false)), ready_event_clone).await;
    });
    rt.block_on(ready_event.wait());
}

fn create_ping_client_tcp_default_config() -> PingClientConfig {
    return PingClientConfig {
        wait_timeout: Duration::from_millis(300),
        time_to_live: None,
        check_disconnect: false,
        wait_before_disconnect: Duration::ZERO,
        disconnect_timeout: Duration::ZERO,
        server_name: None,
        log_tls_key: false,
        alpn_protocol: None,
        use_timer_rtt: false,
    };
}
