use std::time::Duration;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct PingClientPingResultDetails {
    pub actual_local_addr: Option<SocketAddr>,
    pub round_trip_time: Duration,
    pub is_timeout: bool,
}

impl PingClientPingResultDetails {
    pub fn new(
        actual_local_addr: Option<SocketAddr>,
        round_trip_time: Duration,
        is_timeout: bool,
    ) -> PingClientPingResultDetails {
        PingClientPingResultDetails {
            actual_local_addr,
            round_trip_time,
            is_timeout,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PingClientError {
    #[error("preparation failed: {0}")]
    PreparationFailed(Box<dyn std::error::Error + Send>),

    #[error("ping failed: {0}")]
    PingFailed(Box<dyn std::error::Error + Send>),
}

pub type PingClientResult<T, E = PingClientError> = std::result::Result<T, E>;

pub trait PingClient {
    fn protocol(&self) -> &'static str;
    fn ping(&self, source: &SocketAddr, target: &SocketAddr) -> PingClientResult<PingClientPingResultDetails>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ping_clients::ping_client_factory, PingClientConfig};
    use futures_intrusive::sync::ManualResetEvent;
    use std::sync::Arc;
    use std::net::SocketAddr;
    use tide::prelude::*;
    use tide::Request;
    use tokio::runtime::Runtime;
    use socket2::Protocol;
    use pretty_assertions::assert_eq;

    enum ExpectedTestCaseResult {
        Ok,
        Timeout,
        Failed(&'static str),
    }

    struct ExpectedPingClientTestResults {
        timeout_min_time: Duration,
        ping_non_existing_host_result: ExpectedTestCaseResult,
        ping_non_existing_port_result: ExpectedTestCaseResult,
        binding_invalid_source_ip_result: ExpectedTestCaseResult,
        binding_unavailable_source_port_result: ExpectedTestCaseResult,
    }

    #[test]
    fn ping_client_tcp_should_work() {
        let rt = Runtime::new().unwrap();

        let ready_event = Arc::new(ManualResetEvent::new(false));
        let ready_event_clone = ready_event.clone();
        let _server = rt.spawn(async move {
            let mut app = tide::new();
            app.at("/test").get(valid_http_handler);
            let mut listener = app.bind("127.0.0.1:11337").await.unwrap();
            ready_event_clone.set();
            listener.accept().await.unwrap_or_default();
        });
        rt.block_on(ready_event.wait());

        let config = PingClientConfig {
            wait_timeout: Duration::from_millis(300),
            time_to_live: None,
            use_fin_in_tcp_ping: false,
        };
        let mut ping_client = ping_client_factory::new(Protocol::TCP, &config);

        // When connecting to a non existing port, on windows, it will timeout, but on other *nix OS, it will reject the connection.
        let expected_results = ExpectedPingClientTestResults {
            timeout_min_time: Duration::from_millis(200),
            ping_non_existing_host_result: ExpectedTestCaseResult::Timeout,
            ping_non_existing_port_result: if cfg!(windows) {
                ExpectedTestCaseResult::Timeout
            } else {
                ExpectedTestCaseResult::Failed("connection refused")
            },
            binding_invalid_source_ip_result: ExpectedTestCaseResult::Failed("preparation failed: The requested address is not valid in its context. (os error 10049)"),
            binding_unavailable_source_port_result: ExpectedTestCaseResult::Failed("preparation failed: Only one usage of each socket address (protocol/network address/port) is normally permitted. (os error 10048)"),
        };

        run_ping_client_tests(&mut ping_client, &expected_results);
    }

    async fn valid_http_handler(_req: Request<()>) -> tide::Result {
        Ok("It works!".into())
    }

    fn run_ping_client_tests(
        ping_client: &mut Box<dyn PingClient + Send + Sync>,
        expected_results: &ExpectedPingClientTestResults,
    ) {
        // TODO: This is failing on Linux and MAC, need to figure out why.
        if cfg!(windows) {
            ping_client_should_work_when_pinging_good_host(ping_client, expected_results);
        }

        ping_client_should_fail_when_pinging_non_existing_host(ping_client, expected_results);
        ping_client_should_fail_when_pinging_non_existing_port(ping_client, expected_results);
        ping_client_should_fail_when_binding_invalid_source_ip(ping_client, expected_results);
        ping_client_should_fail_when_binding_unavailable_source_port(ping_client, expected_results);
    }

    fn ping_client_should_work_when_pinging_good_host(
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
        );
    }

    fn ping_client_should_fail_when_pinging_non_existing_host(
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
        );
    }

    fn ping_client_should_fail_when_pinging_non_existing_port(
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
        );
    }

    fn ping_client_should_fail_when_binding_invalid_source_ip(
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
        );
    }

    fn ping_client_should_fail_when_binding_unavailable_source_port(
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
        );
    }

    fn ping_client_result_should_be_expected(
        ping_client: &mut Box<dyn PingClient + Send + Sync>,
        source: &SocketAddr,
        target: &SocketAddr,
        timeout_min_time: Duration,
        expected_error: &ExpectedTestCaseResult,
    ) {
        let actual_result = ping_client.ping(source, target);
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
}
