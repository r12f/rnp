use chrono::{DateTime, Utc};
use std::net::{SocketAddr, IpAddr};
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PingResultJsonDto {
    pub utc_time: DateTime<Utc>,
    pub worker_id: u32,
    pub protocol: String,
    pub target_ip: IpAddr,
    pub target_port: u16,
    pub source_ip: IpAddr,
    pub source_port: u16,
    pub is_warmup: bool,
    pub rtt_in_ms: f64,
    pub is_timed_out: bool,
    pub error: String,
    pub handshake_error: String,
}

.write("UTCTime,WorkerId,Protocol,TargetIP,TargetPort,SourceIP,SourcePort,IsWarmup,RTTInMs,IsTimedOut,PingWarning,PreparationError,PingError\n".as_bytes())

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PingResultCsvDto {
    pub utc_time: DateTime<Utc>,
    pub worker_id: u32,
    pub protocol: String,
    pub target: SocketAddr,
    pub source: SocketAddr,
    pub is_warmup: bool,
    pub round_trip_time: Duration,
    pub is_timed_out: bool,
    pub error: String,
    pub handshake_error: String,
}