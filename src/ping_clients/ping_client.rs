use async_trait::async_trait;
use std::net::SocketAddr;
use std::time::Duration;

#[derive(thiserror::Error, Debug)]
pub enum PingClientWarning {
    #[error("{0}")]
    DisconnectFailed(Box<dyn std::error::Error + Send>),

    #[error("{0}")]
    AppHandshakeFailed(Box<dyn std::error::Error + Send>),
}

#[derive(thiserror::Error, Debug)]
pub enum PingClientError {
    #[error("{0}")]
    PreparationFailed(Box<dyn std::error::Error + Send>),

    #[error("{0}")]
    PingFailed(Box<dyn std::error::Error + Send>),
}

#[derive(Debug)]
pub struct PingClientPingResultDetails {
    pub actual_local_addr: Option<SocketAddr>,
    pub round_trip_time: Duration,
    pub is_timeout: bool,
    pub warning: Option<PingClientWarning>,
}

impl PingClientPingResultDetails {
    pub fn new(
        actual_local_addr: Option<SocketAddr>,
        round_trip_time: Duration,
        is_timeout: bool,
        warning: Option<PingClientWarning>,
    ) -> PingClientPingResultDetails {
        PingClientPingResultDetails {
            actual_local_addr,
            round_trip_time,
            is_timeout,
            warning,
        }
    }
}

pub type PingClientResult<T, E = PingClientError> = std::result::Result<T, E>;

#[async_trait]
pub trait PingClient {
    fn protocol(&self) -> &'static str;
    async fn ping(
        &self,
        source: &SocketAddr,
        target: &SocketAddr,
    ) -> PingClientResult<PingClientPingResultDetails>;
}
