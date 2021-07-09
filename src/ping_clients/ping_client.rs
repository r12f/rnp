use socket2::{Protocol, SockAddr};
use std::io;
use std::time::Duration;

#[derive(Debug)]
pub struct PingClientPingResultDetails {
    pub actual_local_addr: Option<SockAddr>,
    pub round_trip_time: Duration,
    pub inner_error: Option<io::Error>,
}
pub type PingClientPingResult = std::result::Result<PingClientPingResultDetails, PingClientPingResultDetails>;

impl PingClientPingResultDetails {
    pub fn new(actual_local_addr: Option<SockAddr>, round_trip_time: Duration, inner_error: Option<io::Error>) -> PingClientPingResultDetails {
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
