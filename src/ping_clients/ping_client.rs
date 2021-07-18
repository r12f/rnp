use std::time::Duration;
use std::net::SocketAddr;
use async_trait::async_trait;

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
    #[error("{0}")]
    PreparationFailed(Box<dyn std::error::Error + Send>),

    #[error("{0}")]
    PingFailed(Box<dyn std::error::Error + Send>),
}

pub type PingClientResult<T, E = PingClientError> = std::result::Result<T, E>;

#[async_trait]
pub trait PingClient {
    fn protocol(&self) -> &'static str;
    async fn ping(&self, source: &SocketAddr, target: &SocketAddr) -> PingClientResult<PingClientPingResultDetails>;
}
