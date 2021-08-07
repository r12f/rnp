use rnp::*;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

pub struct MockPingClient {
    mock_results: Arc<Mutex<Vec<PingClientResult<PingClientPingResultDetails>>>>,
    next_result_index: Arc<Mutex<i32>>,
}

impl MockPingClient {
    pub fn new(mock_results: Vec<PingClientResult<PingClientPingResultDetails>>) -> MockPingClient {
        return MockPingClient {
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

    async fn prepare_ping(&mut self, source: &SocketAddr) -> Result<(), PingClientError> {
        let mut index = self.next_result_index.lock().unwrap();
        *index = *index + 1;
        return Ok(());
    }

    async fn ping(
        &self,
        source: &SocketAddr,
        target: &SocketAddr,
    ) -> PingClientResult<PingClientPingResultDetails> {
        let index = self.next_result_index.lock().unwrap().clone();
        let mock_results = self.mock_results.lock().unwrap();
        return mock_results[index as usize].clone();
    }
}
