use crate::ping_clients::ping_client::PingClientError;
use crate::ping_clients::ping_client::PingClientWarning;
use chrono::{offset::Utc, DateTime};
use contracts::requires;
use std::{net::SocketAddr, time::Duration};

#[derive(Debug)]
pub struct PingResult {
    ping_time: DateTime<Utc>,
    worker_id: u32,
    protocol: &'static str,
    target: SocketAddr,
    source: SocketAddr,
    is_warmup: bool,
    is_succeeded: bool,
    round_trip_time: Duration,
    is_timed_out: bool,
    warning: Option<PingClientWarning>,
    error: Option<PingClientError>,
}

impl PingResult {
    #[requires(is_succeeded -> !is_timed_out && error.is_none())]
    #[requires(warning.is_some() -> is_succeeded)]
    #[requires(!is_succeeded -> (is_timed_out || error.is_some()) && warning.is_none())]
    pub fn new(
        time: &DateTime<Utc>,
        worker_id: u32,
        protocol: &'static str,
        target: SocketAddr,
        source: SocketAddr,
        is_warmup: bool,
        is_succeeded: bool,
        round_trip_time: Duration,
        is_timed_out: bool,
        warning: Option<PingClientWarning>,
        error: Option<PingClientError>,
    ) -> PingResult {
        PingResult {
            ping_time: time.clone(),
            worker_id,
            protocol,
            target,
            source,
            is_warmup,
            is_succeeded,
            round_trip_time,
            is_timed_out,
            warning,
            error,
        }
    }

    pub fn ping_time(&self) -> &DateTime<Utc> {
        &self.ping_time
    }
    pub fn worker_id(&self) -> u32 {
        self.worker_id
    }
    pub fn protocol(&self) -> &'static str {
        self.protocol
    }
    pub fn target(&self) -> SocketAddr {
        self.target
    }
    pub fn source(&self) -> SocketAddr {
        self.source
    }
    pub fn is_warmup(&self) -> bool {
        self.is_warmup
    }
    pub fn is_succeeded(&self) -> bool {
        self.is_succeeded
    }
    pub fn round_trip_time(&self) -> Duration {
        self.round_trip_time
    }
    pub fn is_timed_out(&self) -> bool {
        self.is_timed_out
    }
    pub fn warning(&self) -> &Option<PingClientWarning> {
        &self.warning
    }
    pub fn error(&self) -> &Option<PingClientError> {
        &self.error
    }
    pub fn is_preparation_error(&self) -> bool {
        if let Some(PingClientError::PreparationFailed(_)) = self.error() {
            true
        } else {
            false
        }
    }

    pub fn format_as_console_log(&self) -> String {
        let warmup_sign = if self.is_warmup() { " (warmup)" } else { "" };

        if self.is_timed_out() {
            return format!(
                "Reaching {} {} from {}{} failed: Timed out, RTT = {:.2}ms",
                self.protocol(),
                self.target(),
                self.source(),
                warmup_sign,
                self.round_trip_time().as_micros() as f64 / 1000.0,
            );
        }

        if let Some(error) = self.error() {
            return match error {
                PingClientError::PreparationFailed(e) => {
                    format!(
                        "Unable to perform ping to {} {} from {}{}, because failed preparing to ping: Error = {}",
                        self.protocol(),
                        self.target(),
                        self.source(),
                        warmup_sign,
                        e
                    )
                }

                PingClientError::PingFailed(e) => {
                    format!("Reaching {} {} from {}{} failed: {}", self.protocol(), self.target(), self.source(), warmup_sign, e,)
                }
            };
        }

        if let Some(warning) = self.warning() {
            match warning {
                PingClientWarning::AppHandshakeFailed(e) => {
                    return format!(
                        "Reaching {} {} from {}{} succeeded, but app handshake failed: RTT={:.2}ms, Error = {}",
                        self.protocol(),
                        self.target(),
                        self.source(),
                        warmup_sign,
                        self.round_trip_time().as_micros() as f64 / 1000.0,
                        e,
                    );
                }

                PingClientWarning::DisconnectFailed(e) => {
                    return format!(
                        "Reaching {} {} from {}{} succeeded, but disconnect failed: RTT={:.2}ms, Error = {}",
                        self.protocol(),
                        self.target(),
                        self.source(),
                        warmup_sign,
                        self.round_trip_time().as_micros() as f64 / 1000.0,
                        e,
                    );
                }
            }
        }

        return format!(
            "Reaching {} {} from {}{} succeeded: RTT={:.2}ms",
            self.protocol(),
            self.target(),
            self.source(),
            warmup_sign,
            self.round_trip_time().as_micros() as f64 / 1000.0,
        );
    }

    pub fn format_as_json_string(&self) -> String {
        let preparation_error = self.error().as_ref().map_or(String::from(""), |e| {
            if let PingClientError::PreparationFailed(pe) = e {
                pe.to_string()
            } else {
                String::from("")
            }
        });

        let ping_error =
            self.error().as_ref().map_or(
                String::from(""),
                |e| {
                    if let PingClientError::PingFailed(pe) = e {
                        pe.to_string()
                    } else {
                        String::from("")
                    }
                },
            );

        let handshake_error = self.warning().as_ref().map_or(String::from(""), |w| {
            if let PingClientWarning::AppHandshakeFailed(hw) = w {
                hw.to_string()
            } else {
                String::from("")
            }
        });

        let disconnect_error = self.warning().as_ref().map_or(String::from(""), |w| {
            if let PingClientWarning::DisconnectFailed(dw) = w {
                dw.to_string()
            } else {
                String::from("")
            }
        });

        let json = format!(
            "{{\"utcTime\":\"{:?}\",\"protocol\":\"{}\",\"workerId\":{},\"targetIp\":\"{}\",\"targetPort\":{},\"sourceIp\":\"{}\",\"sourcePort\":{},\"isWarmup\":{},\"isSucceeded\":{},\"rttInMs\":{:.2},\"isTimedOut\":{},\"preparationError\":\"{}\",\"pingError\":\"{}\",\"handshakeError\":\"{}\",\"disconnectError\":\"{}\"}}",
            self.ping_time(),
            self.protocol(),
            self.worker_id(),
            self.target().ip(),
            self.target().port(),
            self.source().ip(),
            self.source().port(),
            self.is_warmup(),
            self.is_succeeded(),
            self.round_trip_time().as_micros() as f64 / 1000.0,
            self.is_timed_out(),
            preparation_error,
            ping_error,
            handshake_error,
            disconnect_error,
        );

        return json;
    }

    pub fn format_as_csv_string(&self) -> String {
        let preparation_error = self.error().as_ref().map_or(String::from(""), |e| {
            if let PingClientError::PreparationFailed(pe) = e {
                pe.to_string()
            } else {
                String::from("")
            }
        });

        let ping_error =
            self.error().as_ref().map_or(
                String::from(""),
                |e| {
                    if let PingClientError::PingFailed(pe) = e {
                        pe.to_string()
                    } else {
                        String::from("")
                    }
                },
            );

        let handshake_error = self.warning().as_ref().map_or(String::from(""), |w| {
            if let PingClientWarning::AppHandshakeFailed(hw) = w {
                hw.to_string()
            } else {
                String::from("")
            }
        });

        let disconnect_error = self.warning().as_ref().map_or(String::from(""), |w| {
            if let PingClientWarning::DisconnectFailed(dw) = w {
                dw.to_string()
            } else {
                String::from("")
            }
        });

        let csv = format!(
            "{:?},{},{},{},{},{},{},{},{},{:.2},{},\"{}\",\"{}\",\"{}\",\"{}\"",
            self.ping_time(),
            self.worker_id(),
            self.protocol(),
            self.target().ip(),
            self.target().port(),
            self.source().ip(),
            self.source().port(),
            self.is_warmup(),
            self.is_succeeded(),
            self.round_trip_time().as_micros() as f64 / 1000.0,
            self.is_timed_out(),
            preparation_error,
            ping_error,
            handshake_error,
            disconnect_error,
        );

        return csv;
    }
}

#[cfg(test)]
mod tests {
    use crate::ping_result::PingResult;
    use crate::rnp_test_common;
    use chrono::Utc;
    use pretty_assertions::assert_eq;
    use std::net::SocketAddr;
    use std::time::Duration;

    #[test]
    fn new_ping_result_should_work() {
        let r = PingResult::new(
            &Utc::now(),
            1,
            "TCP",
            "1.2.3.4:443".parse().unwrap(),
            "5.6.7.8:8080".parse().unwrap(),
            true,
            true,
            Duration::from_millis(10),
            false,
            None,
            None,
        );

        assert_eq!(1, r.worker_id());
        assert_eq!("TCP", r.protocol());
        assert_eq!("1.2.3.4:443".parse::<SocketAddr>().unwrap(), r.target());
        assert_eq!("5.6.7.8:8080".parse::<SocketAddr>().unwrap(), r.source());
        assert!(r.is_warmup());
        assert!(r.is_succeeded());
        assert_eq!(Duration::from_millis(10), r.round_trip_time());
        assert!(r.error().is_none());
        assert!(r.warning().is_none());
    }

    #[test]
    fn format_ping_result_as_log_should_work() {
        let results = rnp_test_common::generate_ping_result_test_samples();
        assert_eq!(
            vec![
                "Reaching TCP 1.2.3.4:443 from 5.6.7.8:8080 (warmup) succeeded: RTT=10.00ms",
                "Reaching TCP 1.2.3.4:443 from 5.6.7.8:8080 failed: Timed out, RTT = 1000.00ms",
                "Reaching TCP 1.2.3.4:443 from 5.6.7.8:8080 succeeded, but app handshake failed: RTT=20.00ms, Error = connect aborted",
                "Reaching TCP 1.2.3.4:443 from 5.6.7.8:8080 succeeded, but disconnect failed: RTT=20.00ms, Error = disconnect timeout",
                "Reaching TCP 1.2.3.4:443 from 5.6.7.8:8080 failed: connect failed",
                "Unable to perform ping to TCP 1.2.3.4:443 from 5.6.7.8:8080, because failed preparing to ping: Error = address in use",
            ],
            results.into_iter().map(|x| x.format_as_console_log()).collect::<Vec<String>>()
        );
    }

    #[test]
    fn format_ping_result_as_json_should_work() {
        let results = rnp_test_common::generate_ping_result_test_samples();
        assert_eq!(
            vec![
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIp\":\"1.2.3.4\",\"targetPort\":443,\"sourceIp\":\"5.6.7.8\",\"sourcePort\":8080,\"isWarmup\":true,\"isSucceeded\":true,\"rttInMs\":10.00,\"isTimedOut\":false,\"preparationError\":\"\",\"pingError\":\"\",\"handshakeError\":\"\",\"disconnectError\":\"\"}",
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIp\":\"1.2.3.4\",\"targetPort\":443,\"sourceIp\":\"5.6.7.8\",\"sourcePort\":8080,\"isWarmup\":false,\"isSucceeded\":false,\"rttInMs\":1000.00,\"isTimedOut\":true,\"preparationError\":\"\",\"pingError\":\"\",\"handshakeError\":\"\",\"disconnectError\":\"\"}",
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIp\":\"1.2.3.4\",\"targetPort\":443,\"sourceIp\":\"5.6.7.8\",\"sourcePort\":8080,\"isWarmup\":false,\"isSucceeded\":true,\"rttInMs\":20.00,\"isTimedOut\":false,\"preparationError\":\"\",\"pingError\":\"\",\"handshakeError\":\"connect aborted\",\"disconnectError\":\"\"}",
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIp\":\"1.2.3.4\",\"targetPort\":443,\"sourceIp\":\"5.6.7.8\",\"sourcePort\":8080,\"isWarmup\":false,\"isSucceeded\":true,\"rttInMs\":20.00,\"isTimedOut\":false,\"preparationError\":\"\",\"pingError\":\"\",\"handshakeError\":\"\",\"disconnectError\":\"disconnect timeout\"}",
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIp\":\"1.2.3.4\",\"targetPort\":443,\"sourceIp\":\"5.6.7.8\",\"sourcePort\":8080,\"isWarmup\":false,\"isSucceeded\":false,\"rttInMs\":0.00,\"isTimedOut\":false,\"preparationError\":\"\",\"pingError\":\"connect failed\",\"handshakeError\":\"\",\"disconnectError\":\"\"}",
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIp\":\"1.2.3.4\",\"targetPort\":443,\"sourceIp\":\"5.6.7.8\",\"sourcePort\":8080,\"isWarmup\":false,\"isSucceeded\":false,\"rttInMs\":0.00,\"isTimedOut\":false,\"preparationError\":\"address in use\",\"pingError\":\"\",\"handshakeError\":\"\",\"disconnectError\":\"\"}",
            ],
            results.into_iter().map(|x| x.format_as_json_string()).collect::<Vec<String>>()
        );
    }

    #[test]
    fn format_ping_result_as_csv_should_work() {
        let results = rnp_test_common::generate_ping_result_test_samples();
        assert_eq!(
            vec![
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,true,true,10.00,false,\"\",\"\",\"\",\"\"",
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,false,false,1000.00,true,\"\",\"\",\"\",\"\"",
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,false,true,20.00,false,\"\",\"\",\"connect aborted\",\"\"",
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,false,true,20.00,false,\"\",\"\",\"\",\"disconnect timeout\"",
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,false,false,0.00,false,\"\",\"connect failed\",\"\",\"\"",
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,false,false,0.00,false,\"address in use\",\"\",\"\",\"\"",
            ],
            results.into_iter().map(|x| x.format_as_csv_string()).collect::<Vec<String>>()
        );
    }
}
