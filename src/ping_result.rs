use socket2::Protocol;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use chrono::DateTime;
use chrono::offset::Utc;
use crate::rnp_utils;

#[derive(Debug)]
pub struct PingResult {
    utctime: DateTime<Utc>,
    worker_id: u32,
    protocol: Protocol,
    target: SocketAddr,
    source: SocketAddr,
    round_trip_time: Duration,
    error: Option<io::Error>,
}

impl PingResult {
    pub fn new(
        time: &DateTime<Utc>,
        worker_id: u32,
        protocol: Protocol,
        target: SocketAddr,
        source: SocketAddr,
        round_trip_time: Duration,
        error: Option<io::Error>,
    ) -> PingResult {
        PingResult {
            utctime: time.clone(),
            worker_id,
            protocol,
            target,
            source,
            round_trip_time,
            error,
        }
    }

    pub fn utc_time(&self) -> &DateTime<Utc> {
        &self.utctime
    }
    pub fn worker_id(&self) -> u32 {
        self.worker_id
    }
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }
    pub fn protocol_string(&self) -> &str { rnp_utils::format_protocol(self.protocol()) }
    pub fn target(&self) -> SocketAddr {
        self.target
    }
    pub fn source(&self) -> SocketAddr {
        self.source
    }
    pub fn round_trip_time(&self) -> Duration {
        self.round_trip_time
    }
    pub fn error(&self) -> &Option<io::Error> {
        &self.error
    }

    pub fn format_as_console_log(&self) -> String {
        match self.error() {
            Some(e) if e.kind() == io::ErrorKind::TimedOut => {
                return format!(
                    "Reaching {} {} from {} failed: Timed out, RTT = {:.2}ms",
                    self.protocol_string(),
                    self.target(),
                    self.source(),
                    self.round_trip_time().as_micros() as f64 / 1000.0,
                );
            },
            Some(e) => {
                return format!(
                    "Reaching {} {} from {} failed: {}",
                    self.protocol_string(),
                    self.target(),
                    self.source(),
                    e,
                );
            },
            _ => {
                return format!(
                    "Reaching {} {} from {} succeeded: RTT={:.2}ms",
                    self.protocol_string(),
                    self.target(),
                    self.source(),
                    self.round_trip_time().as_micros() as f64 / 1000.0,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ping_result::PingResult;
    use socket2::Protocol;
    use std::net::SocketAddr;
    use std::time::Duration;
    use chrono::Utc;

    #[test]
    fn new_ping_result_should_work() {
        let r = PingResult::new(
            &Utc::now(),
            1,
            Protocol::TCP,
            "1.2.3.4:443".parse().unwrap(),
            "5.6.7.8:8080".parse().unwrap(),
            Duration::from_millis(10),
            None,
        );

        assert_eq!(1, r.worker_id());
        assert_eq!(Protocol::TCP, r.protocol());
        assert_eq!("TCP", r.protocol_string());
        assert_eq!("1.2.3.4:443".parse::<SocketAddr>().unwrap(), r.target());
        assert_eq!("5.6.7.8:8080".parse::<SocketAddr>().unwrap(), r.source());
        assert_eq!(Duration::from_millis(10), r.round_trip_time());
        assert!(r.error().is_none());
    }
}
