use socket2::{Protocol, SockAddr};
use std::io;
use std::time::Duration;

#[derive(Debug)]
pub struct PingClientPingResultDetails {
    pub actual_local_addr: Option<SockAddr>,
    pub round_trip_time: Duration,
    pub inner_error: Option<io::Error>,
}
pub type PingClientPingResult =
    std::result::Result<PingClientPingResultDetails, PingClientPingResultDetails>;

impl PingClientPingResultDetails {
    pub fn new(
        actual_local_addr: Option<SockAddr>,
        round_trip_time: Duration,
        inner_error: Option<io::Error>,
    ) -> PingClientPingResultDetails {
        PingClientPingResultDetails {
            actual_local_addr,
            round_trip_time,
            inner_error,
        }
    }
}

impl From<io::Error> for PingClientPingResultDetails {
    fn from(e: io::Error) -> PingClientPingResultDetails {
        PingClientPingResultDetails {
            actual_local_addr: None,
            round_trip_time: Duration::from_secs(0),
            inner_error: Some(e),
        }
    }
}

pub trait PingClient {
    fn protocol(&self) -> Protocol;

    fn prepare(&mut self) {}
    fn ping(&self, source: &SockAddr, target: &SockAddr) -> PingClientPingResult;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ping_clients::ping_client_factory, PingClientConfig};
    use futures_intrusive::sync::ManualResetEvent;
    use std::{io, net::SocketAddr};
    use tide::prelude::*;
    use tide::Request;
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    struct ExpectedPingClientTestResults {
        timeout_min_time: Duration,
        ping_non_existing_host_error: io::ErrorKind,
        ping_non_existing_port_error: io::ErrorKind,
        binding_invalid_source_ip_error: io::ErrorKind,
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
        };
        let ping_client = ping_client_factory::new(Protocol::TCP, &config);

        // When connecting to a non existing port, on windows, it will timeout, but on other *nix OS, it will reject the connection.
        let expected_results = ExpectedPingClientTestResults {
            timeout_min_time: Duration::from_millis(200),
            ping_non_existing_host_error: io::ErrorKind::TimedOut,
            ping_non_existing_port_error: if cfg!(windows) {
                io::ErrorKind::TimedOut
            } else {
                io::ErrorKind::ConnectionRefused
            },
            binding_invalid_source_ip_error: io::ErrorKind::AddrNotAvailable,
        };

        run_ping_client_tests(&ping_client, &expected_results);
    }

    async fn valid_http_handler(_req: Request<()>) -> tide::Result {
        Ok("It works!".into())
    }

    fn run_ping_client_tests(
        ping_client: &Box<dyn PingClient + Send + Sync>,
        expected_results: &ExpectedPingClientTestResults,
    ) {
        // TODO: This is failing on Linux and MAC, need to figure out why.
        if cfg!(windows) {
            ping_client_should_work_when_pinging_good_host(ping_client, expected_results);
        }

        ping_client_should_fail_when_pinging_non_existing_host(ping_client, expected_results);
        ping_client_should_fail_when_pinging_non_existing_port(ping_client, expected_results);
        ping_client_should_fail_when_binding_invalid_source_ip(ping_client, expected_results);
    }

    fn ping_client_should_work_when_pinging_good_host(
        ping_client: &Box<dyn PingClient + Send + Sync>,
        expected_results: &ExpectedPingClientTestResults,
    ) {
        let source = SockAddr::from("0.0.0.0:0".parse::<SocketAddr>().unwrap());
        let target = SockAddr::from("127.0.0.1:3389".parse::<SocketAddr>().unwrap());
        let result = ping_client.ping(&source, &target);
        ping_client_result_should_be_expected(&result, None, expected_results.timeout_min_time);
    }

    fn ping_client_should_fail_when_pinging_non_existing_host(
        ping_client: &Box<dyn PingClient + Send + Sync>,
        expected_results: &ExpectedPingClientTestResults,
    ) {
        let source = SockAddr::from("0.0.0.0:0".parse::<SocketAddr>().unwrap());
        let target = SockAddr::from("1.1.1.1:11111".parse::<SocketAddr>().unwrap());
        let result = ping_client.ping(&source, &target);
        ping_client_result_should_be_expected(
            &result,
            Some(expected_results.ping_non_existing_host_error),
            expected_results.timeout_min_time,
        );
    }

    fn ping_client_should_fail_when_pinging_non_existing_port(
        ping_client: &Box<dyn PingClient + Send + Sync>,
        expected_results: &ExpectedPingClientTestResults,
    ) {
        let source = SockAddr::from("0.0.0.0:0".parse::<SocketAddr>().unwrap());
        let target = SockAddr::from("127.0.0.1:56789".parse::<SocketAddr>().unwrap());
        let result = ping_client.ping(&source, &target);
        ping_client_result_should_be_expected(
            &result,
            Some(expected_results.ping_non_existing_port_error),
            expected_results.timeout_min_time,
        );
    }

    fn ping_client_should_fail_when_binding_invalid_source_ip(
        ping_client: &Box<dyn PingClient + Send + Sync>,
        expected_results: &ExpectedPingClientTestResults,
    ) {
        let source = SockAddr::from("1.1.1.1:1111".parse::<SocketAddr>().unwrap());
        let target = SockAddr::from("127.0.0.1:56789".parse::<SocketAddr>().unwrap());
        let result = ping_client.ping(&source, &target);
        ping_client_result_should_be_expected(
            &result,
            Some(expected_results.binding_invalid_source_ip_error),
            expected_results.timeout_min_time,
        );
    }

    fn ping_client_result_should_be_expected(
        result: &PingClientPingResult,
        expected_error: Option<io::ErrorKind>,
        timeout_min_time: Duration,
    ) {
        match expected_error {
            None => {
                assert!(result.is_ok());
            }

            Some(error) => {
                assert!(result.is_err());
                assert!(result.as_ref().err().is_some());

                let actual_error_details = result.as_ref().err().unwrap();
                assert!(actual_error_details.inner_error.is_some());

                let actual_error_kind = actual_error_details.inner_error.as_ref().unwrap().kind();
                assert_eq!(actual_error_kind, error);

                if error == io::ErrorKind::TimedOut {
                    assert!(actual_error_details.round_trip_time > timeout_min_time);
                } else {
                    assert_eq!(0, actual_error_details.round_trip_time.as_micros());
                }
            }
        }
    }
}
