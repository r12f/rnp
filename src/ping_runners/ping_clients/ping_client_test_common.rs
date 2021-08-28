use crate::*;
use pretty_assertions::assert_eq;
use std::net::SocketAddr;
use std::time::Duration;

pub enum ExpectedTestCaseResult {
    Ok,
    Timeout,
    Failed(&'static str),
}

pub async fn ping_client_should_work_when_pinging_good_host(ping_client: &mut Box<dyn PingClient + Send + Sync>, server_address: &SocketAddr) {
    let target = server_address.clone();
    if target.port() == 0 {
        return;
    }

    let source = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(ping_client, &source, &target, Duration::from_millis(200), &ExpectedTestCaseResult::Ok).await;
}

pub async fn ping_client_should_fail_when_binding_unavailable_source_port(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    source: &SocketAddr,
    expected_result: &ExpectedTestCaseResult,
) {
    let target = "127.0.0.1:56789".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(ping_client, &source, &target, Duration::from_millis(200), &expected_result).await;
}

pub async fn ping_client_should_fail_when_pinging_non_existing_host(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    expected_result: &ExpectedTestCaseResult,
) {
    let source = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let target = "1.1.1.1:11111".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(ping_client, &source, &target, Duration::from_millis(200), expected_result).await;
}

pub async fn ping_client_should_fail_when_pinging_non_existing_port(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    expected_result: &ExpectedTestCaseResult,
) {
    let source = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let target = "127.0.0.1:56789".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(ping_client, &source, &target, Duration::from_millis(200), expected_result).await;
}

pub async fn ping_client_should_fail_when_binding_invalid_source_ip(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    expected_result: &ExpectedTestCaseResult,
) {
    let source = "1.1.1.1:1111".parse::<SocketAddr>().unwrap();
    let target = "127.0.0.1:56789".parse::<SocketAddr>().unwrap();
    ping_client_result_should_be_expected(ping_client, &source, &target, Duration::from_millis(200), expected_result).await;
}

pub async fn ping_client_result_should_be_expected(
    ping_client: &mut Box<dyn PingClient + Send + Sync>,
    source: &SocketAddr,
    target: &SocketAddr,
    timeout_min_time: Duration,
    expected_result: &ExpectedTestCaseResult,
) {
    let actual_result = ping_client.ping(source, target).await;
    let ping_result_string = format!("Ping result: {:?}", actual_result);
    tracing::info!("{}", ping_result_string);

    match expected_result {
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
