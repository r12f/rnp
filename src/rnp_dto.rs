use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PingResultDto {
    pub utc_time: DateTime<Utc>,
    pub worker_id: u32,
    pub protocol: String,
    pub target_ip: IpAddr,
    pub target_port: u16,
    pub source_ip: IpAddr,
    pub source_port: u16,
    pub is_warmup: bool,
    pub is_succeeded: bool,
    pub rtt_in_ms: f64,
    pub is_timed_out: bool,
    pub preparation_error: String,
    pub ping_error: String,
    pub handshake_error: String,
    pub disconnect_error: String,
}

impl PingResultDto {
    pub fn to_console_log(&self) -> String {
        let warmup_sign = if self.is_warmup { " (warmup)" } else { "" };

        if self.is_timed_out {
            return format!(
                "Reaching {} {}:{} from {}:{}{} failed: Timed out, RTT = {:.2}ms",
                self.protocol, self.target_ip, self.target_port, self.source_ip, self.source_port, warmup_sign, self.rtt_in_ms,
            );
        }

        if !self.preparation_error.is_empty() {
            return format!(
                "Unable to perform ping to {} {}:{} from {}:{}{}, because failed preparing to ping: Error = {}",
                self.protocol, self.target_ip, self.target_port, self.source_ip, self.source_port, warmup_sign, self.preparation_error,
            );
        }

        if !self.ping_error.is_empty() {
            return format!(
                "Reaching {} {}:{} from {}:{}{} failed: {}",
                self.protocol, self.target_ip, self.target_port, self.source_ip, self.source_port, warmup_sign, self.ping_error,
            );
        }

        if !self.handshake_error.is_empty() {
            return format!(
                "Reaching {} {}:{} from {}:{}{} succeeded, but app handshake failed: RTT={:.2}ms, Error = {}",
                self.protocol, self.target_ip, self.target_port, self.source_ip, self.source_port, warmup_sign, self.rtt_in_ms, self.handshake_error,
            );
        }

        if !self.disconnect_error.is_empty() {
            return format!(
                "Reaching {} {}:{} from {}:{}{} succeeded, but disconnect failed: RTT={:.2}ms, Error = {}",
                self.protocol, self.target_ip, self.target_port, self.source_ip, self.source_port, warmup_sign, self.rtt_in_ms, self.disconnect_error,
            );
        }

        return format!(
            "Reaching {} {}:{} from {}:{}{} succeeded: RTT={:.2}ms",
            self.protocol, self.target_ip, self.target_port, self.source_ip, self.source_port, warmup_sign, self.rtt_in_ms,
        );
    }

    pub fn to_json_lite(&self) -> String {
        format!(
            "{{\"UtcTime\":\"{:?}\",\"WorkerId\":{},\"Protocol\":\"{}\",\"TargetIp\":\"{}\",\"TargetPort\":{},\"SourceIp\":\"{}\",\"SourcePort\":{},\"IsWarmup\":{},\"IsSucceeded\":{},\"RttInMs\":{:.2},\"IsTimedOut\":{},\"PreparationError\":\"{}\",\"PingError\":\"{}\",\"HandshakeError\":\"{}\",\"DisconnectError\":\"{}\"}}",
            self.utc_time,
            self.worker_id,
            self.protocol,
            self.target_ip,
            self.target_port,
            self.source_ip,
            self.source_port,
            self.is_warmup,
            self.is_succeeded,
            self.rtt_in_ms,
            self.is_timed_out,
            self.preparation_error,
            self.ping_error,
            self.handshake_error,
            self.disconnect_error,
        )
    }

    pub fn to_csv_lite(&self) -> String {
        format!(
            "{:?},{},{},{},{},{},{},{},{},{:.2},{},\"{}\",\"{}\",\"{}\",\"{}\"",
            self.utc_time,
            self.worker_id,
            self.protocol,
            self.target_ip,
            self.target_port,
            self.source_ip,
            self.source_port,
            self.is_warmup,
            self.is_succeeded,
            self.rtt_in_ms,
            self.is_timed_out,
            self.preparation_error,
            self.ping_error,
            self.handshake_error,
            self.disconnect_error,
        )
    }
}
