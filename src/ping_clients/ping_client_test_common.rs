use crate::*;
use pretty_assertions::assert_eq;
use std::net::SocketAddr;
use std::time::Duration;

pub enum ExpectedTestCaseResult {
    Ok,
    Timeout,
    Failed(&'static str),
}

pub struct ExpectedPingClientTestResults {
    pub timeout_min_time: Duration,
    pub ping_non_existing_host_result: ExpectedTestCaseResult,
    pub ping_non_existing_port_result: ExpectedTestCaseResult,
    pub binding_invalid_source_ip_result: ExpectedTestCaseResult,
    pub binding_unavailable_source_port_result: ExpectedTestCaseResult,
}

pub async fn run_ping_client_tests(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    expected_results: &ExpectedPingClientTestResults,
) {
    // TODO: This is failing on Linux and MAC, need to figure out why.
    if cfg!(windows) {
        ping_client_should_work_when_pinging_good_host(ping_client, expected_results).await;
    }

    ping_client_should_fail_when_pinging_non_existing_host(ping_client, expected_results).await;
    ping_client_should_fail_when_pinging_non_existing_port(ping_client, expected_results).await;
    ping_client_should_fail_when_binding_invalid_source_ip(ping_client, expected_results).await;
    ping_client_should_fail_when_binding_unavailable_source_port(ping_client, expected_results)
        .await;
}

async fn ping_client_should_work_when_pinging_good_host(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    expected_results: &ExpectedPingClientTestResults,
) {
    let source = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let target = "127.0.0.1:3389".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(
        ping_client,
        &source,
        &target,
        expected_results.timeout_min_time,
        &ExpectedTestCaseResult::Ok,
    )
    .await;
}

async fn ping_client_should_fail_when_pinging_non_existing_host(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    expected_results: &ExpectedPingClientTestResults,
) {
    let source = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let target = "1.1.1.1:11111".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(
        ping_client,
        &source,
        &target,
        expected_results.timeout_min_time,
        &expected_results.ping_non_existing_host_result,
    )
    .await;
}

async fn ping_client_should_fail_when_pinging_non_existing_port(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    expected_results: &ExpectedPingClientTestResults,
) {
    let source = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let target = "127.0.0.1:56789".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(
        ping_client,
        &source,
        &target,
        expected_results.timeout_min_time,
        &expected_results.ping_non_existing_port_result,
    )
    .await;
}

async fn ping_client_should_fail_when_binding_invalid_source_ip(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    expected_results: &ExpectedPingClientTestResults,
) {
    let source = "1.1.1.1:1111".parse::<SocketAddr>().unwrap();
    let target = "127.0.0.1:56789".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(
        ping_client,
        &source,
        &target,
        expected_results.timeout_min_time,
        &expected_results.binding_invalid_source_ip_result,
    )
    .await;
}

async fn ping_client_should_fail_when_binding_unavailable_source_port(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    expected_results: &ExpectedPingClientTestResults,
) {
    let source = "127.0.0.1:11337".parse::<SocketAddr>().unwrap();
    let target = "127.0.0.1:56789".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(
        ping_client,
        &source,
        &target,
        expected_results.timeout_min_time,
        &expected_results.binding_unavailable_source_port_result,
    )
    .await;
}

async fn ping_client_result_should_be_expected(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    source: &SocketAddr,
    target: &SocketAddr,
    timeout_min_time: Duration,
    expected_error: &ExpectedTestCaseResult,
) {
    let actual_result = ping_client.ping(source, target).await;
    match expected_error {
        ExpectedTestCaseResult::Ok => {
            assert!(actual_result.is_ok());
            assert!(!(actual_result.as_ref().ok().unwrap().is_timeout));
            return;
        }

        ExpectedTestCaseResult::Timeout => {
            assert!(actual_result.is_ok());
            assert!(actual_result.as_ref().ok().unwrap().is_timeout);
            assert!(actual_result.as_ref().ok().unwrap().round_trip_time > timeout_min_time)
        }

        ExpectedTestCaseResult::Failed(e) => {
            assert!(actual_result.is_err());
            assert!(actual_result.as_ref().err().is_some());

            // On windows, we will check the detailed failure.
            if cfg!(windows) {
                let actual_error: &str = &actual_result.as_ref().err().unwrap().to_string();
                assert_eq!(*e, actual_error);
            }
        }
    }
}
