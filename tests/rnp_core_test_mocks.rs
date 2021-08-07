use async_trait::async_trait;
use rnp::{
    PingClient, PingClientConfig, PingClientError, PingClientPingResultDetails, PingClientResult,
    PingClientWarning, PingResult, PingResultProcessor,
};
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum MockPingClientResult {
    Success(Duration),
    Timeout,
    PreparationFailed,
    PingFailed,
    AppHandshakeFailed(Duration),
    DisconnectFailed(Duration),
}

pub struct MockPingClient {
    config: PingClientConfig,
    mock_results: Arc<Mutex<Vec<MockPingClientResult>>>,
    next_result_index: Arc<Mutex<i32>>,
}

impl MockPingClient {
    pub fn new(
        config: &PingClientConfig,
        mock_results: Vec<MockPingClientResult>,
    ) -> MockPingClient {
        return MockPingClient {
            config: config.clone(),
            mock_results: Arc::new(Mutex::new(mock_results)),
            next_result_index: Arc::new(Mutex::new(-1)),
        };
    }
}

#[async_trait]
impl PingClient for MockPingClient {
    fn protocol(&self) -> &'static str {
        "TCP"
    }

    async fn prepare_ping(&mut self, _: &SocketAddr) -> Result<(), PingClientError> {
        let mut index = self.next_result_index.lock().unwrap();

        *index = *index + 1;
        if *index >= self.mock_results.lock().unwrap().len() as i32 {
            *index = 0;
        }

        return Ok(());
    }

    async fn ping(
        &self,
        _: &SocketAddr,
        _: &SocketAddr,
    ) -> PingClientResult<PingClientPingResultDetails> {
        let index = self.next_result_index.lock().unwrap().clone();
        let mock_result: MockPingClientResult;
        {
            let mock_results = self.mock_results.lock().unwrap();
            mock_result = mock_results[index as usize].clone();
        }

        match mock_result {
            MockPingClientResult::Success(rtt) => {
                return Ok(PingClientPingResultDetails::new(None, rtt, false, None))
            }
            MockPingClientResult::Timeout => {
                return Ok(PingClientPingResultDetails::new(
                    None,
                    self.config.wait_timeout,
                    true,
                    None,
                ))
            }
            MockPingClientResult::PreparationFailed => {
                return Err(PingClientError::PreparationFailed(Box::new(
                    io::Error::from(io::ErrorKind::AddrNotAvailable),
                )))
            }
            MockPingClientResult::PingFailed => {
                return Err(PingClientError::PingFailed(Box::new(io::Error::from(
                    io::ErrorKind::ConnectionRefused,
                ))))
            }
            MockPingClientResult::AppHandshakeFailed(rtt) => {
                return Ok(PingClientPingResultDetails::new(
                    None,
                    rtt,
                    false,
                    Some(PingClientWarning::AppHandshakeFailed(Box::new(
                        io::Error::from(io::ErrorKind::PermissionDenied),
                    ))),
                ))
            }
            MockPingClientResult::DisconnectFailed(rtt) => {
                return Ok(PingClientPingResultDetails::new(
                    None,
                    rtt,
                    false,
                    Some(PingClientWarning::DisconnectFailed(Box::new(
                        io::Error::from(io::ErrorKind::ConnectionAborted),
                    ))),
                ))
            }
        }
    }
}

pub struct MockPingResultProcessor {
    results: Arc<Mutex<Vec<MockPingClientResult>>>,
}

impl MockPingResultProcessor {
    pub fn new(results: Arc<Mutex<Vec<MockPingClientResult>>>) -> MockPingResultProcessor {
        return MockPingResultProcessor {
            results,
        };
    }
}

impl PingResultProcessor for MockPingResultProcessor {
    fn name(&self) -> &'static str {
        return "MockPingResultProcessor";
    }

    fn process_ping_result(&mut self, ping_result: &PingResult) {
        let mut results = self.results.lock().unwrap();
        if ping_result.is_timed_out() {
            results.push(MockPingClientResult::Timeout);
            return;
        }

        if let Some(warning) = ping_result.warning() {
            match warning {
                PingClientWarning::AppHandshakeFailed(_) => results.push(
                    MockPingClientResult::AppHandshakeFailed(ping_result.round_trip_time()),
                ),
                PingClientWarning::DisconnectFailed(_) => results.push(
                    MockPingClientResult::DisconnectFailed(ping_result.round_trip_time()),
                ),
            }
            return;
        }

        if let Some(error) = ping_result.error() {
            match error {
                PingClientError::PreparationFailed(_) => {
                    results.push(MockPingClientResult::PreparationFailed)
                }
                PingClientError::PingFailed(_) => results.push(MockPingClientResult::PingFailed),
            }
            return;
        }

        if ping_result.is_succeeded() {
            results.push(MockPingClientResult::Success(ping_result.round_trip_time()));
            return;
        }
    }
}
