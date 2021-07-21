use crate::ping_clients::ping_client::PingClientError::{self, PingFailed, PreparationFailed};
use chrono::{offset::Utc, DateTime};
use std::{net::SocketAddr, time::Duration};
use contracts::requires;

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
    error: Option<PingClientError>,
    handshake_error: Option<Box<dyn std::error::Error + Send>>,
}

impl PingResult {
    #[requires(is_succeeded -> !is_timed_out && error.is_none())]
    #[requires(handshake_error.is_some() -> is_succeeded)]
    #[requires(!is_succeeded -> (is_timed_out || error.is_some()) && handshake_error.is_none())]
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
        error: Option<PingClientError>,
        handshake_error: Option<Box<dyn std::error::Error + Send>>,
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
            error,
            handshake_error,
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
    pub fn handshake_error(&self) -> &Option<Box<dyn std::error::Error + Send>> {
        &self.handshake_error
    }
    pub fn error(&self) -> &Option<PingClientError> {
        &self.error
    }
    pub fn is_preparation_error(&self) -> bool {
        if let Some(PreparationFailed(_)) = self.error() {
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
                PreparationFailed(e) => {
                    format!(
                            "Unable to perform ping to {} {} from {}{}, because failed preparing to ping: Error = {}",
                            self.protocol(),
                            self.target(),
                            self.source(),
                            warmup_sign,
                            e)
                }

                PingFailed(e) => {
                    format!(
                        "Reaching {} {} from {}{} failed: {}",
                        self.protocol(),
                        self.target(),
                        self.source(),
                        warmup_sign,
                        e,
                    )
                }
            };
        }

        if let Some(handshake_error) = self.handshake_error() {
            return format!(
                "Reaching {} {} from {}{} succeeded, but handshake failed: RTT={:.2}ms, Error = {}",
                self.protocol(),
                self.target(),
                self.source(),
                warmup_sign,
                self.round_trip_time().as_micros() as f64 / 1000.0,
                handshake_error,
            );
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
            if let PreparationFailed(pe) = e { pe.to_string() } else { String::from("") }
        });

        let ping_error = self.error().as_ref().map_or(String::from(""), |e| {
            if let PingFailed(pe) = e { pe.to_string() } else { String::from("") }
        });

        let handshake_error = self.handshake_error().as_ref().map_or(String::from(""), |e| {
            e.to_string()
        });

        let json = format!(
            "{{\"utcTime\":\"{:?}\",\"protocol\":\"{}\",\"workerId\":{},\"targetIP\":\"{}\",\"targetPort\":\"{}\",\"sourceIP\":\"{}\",\"sourcePort\":\"{}\",\"isWarmup\":\"{}\",\"roundTripTimeInMs\":{:.2},\"isTimedOut\":\"{}\",\"preparationError\":\"{}\",\"pingError\":\"{}\",\"handshakeError\":\"{}\"}}",
            self.ping_time(),
            self.protocol(),
            self.worker_id(),
            self.target().ip(),
            self.target().port(),
            self.source().ip(),
            self.source().port(),
            self.is_warmup(),
            self.round_trip_time().as_micros() as f64 / 1000.0,
            self.is_timed_out(),
            preparation_error,
            ping_error,
            handshake_error,
        );

        return json;
    }

    pub fn format_as_csv_string(&self) -> String {
        let preparation_error = self.error().as_ref().map_or(String::from(""), |e| {
            if let PreparationFailed(pe) = e { pe.to_string() } else { String::from("") }
        });

        let ping_error = self.error().as_ref().map_or(String::from(""), |e| {
            if let PingFailed(pe) = e { pe.to_string() } else { String::from("") }
        });

        let handshake_error = self.handshake_error().as_ref().map_or(String::from(""), |e| {
            e.to_string()
        });

        let csv = format!(
            "{:?},{},{},{},{},{},{},{},{:.2},{},\"{}\",\"{}\",\"{}\"",
            self.ping_time(),
            self.worker_id(),
            self.protocol(),
            self.target().ip(),
            self.target().port(),
            self.source().ip(),
            self.source().port(),
            self.is_warmup(),
            self.round_trip_time().as_micros() as f64 / 1000.0,
            self.is_timed_out(),
            preparation_error,
            ping_error,
            handshake_error,
        );

        return csv;
    }
}

#[cfg(test)]
mod tests {
    use crate::ping_result::PingResult;
    use chrono::Utc;
    use pretty_assertions::assert_eq;
    use std::net::SocketAddr;
    use std::time::Duration;
    use crate::rnp_test_utils;

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
        assert!(r.handshake_error().is_none());
    }

    #[test]
    fn format_ping_result_as_log_should_work() {
        let results = rnp_test_utils::generate_ping_result_test_samples();
        assert_eq!(
            vec![
                "Reaching TCP 1.2.3.4:443 from 5.6.7.8:8080 (warmup) succeeded: RTT=10.00ms",
                "Reaching TCP 1.2.3.4:443 from 5.6.7.8:8080 failed: Timed out, RTT = 1000.00ms",
                "Reaching TCP 1.2.3.4:443 from 5.6.7.8:8080 succeeded, but handshake failed: RTT=20.00ms, Error = connect aborted",
                "Reaching TCP 1.2.3.4:443 from 5.6.7.8:8080 failed: connect failed",
                "Unable to perform ping to TCP 1.2.3.4:443 from 5.6.7.8:8080, because failed preparing to ping: Error = address in use",
            ],
            results
                .into_iter()
                .map(|x| x.format_as_console_log())
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn format_ping_result_as_json_should_work() {
        let results = rnp_test_utils::generate_ping_result_test_samples();
        assert_eq!(
            vec![
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIP\":\"1.2.3.4\",\"targetPort\":\"443\",\"sourceIP\":\"5.6.7.8\",\"sourcePort\":\"8080\",\"isWarmup\":\"true\",\"roundTripTimeInMs\":10.00,\"isTimedOut\":\"false\",\"preparationError\":\"\",\"pingError\":\"\",\"handshakeError\":\"\"}",
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIP\":\"1.2.3.4\",\"targetPort\":\"443\",\"sourceIP\":\"5.6.7.8\",\"sourcePort\":\"8080\",\"isWarmup\":\"false\",\"roundTripTimeInMs\":1000.00,\"isTimedOut\":\"true\",\"preparationError\":\"\",\"pingError\":\"\",\"handshakeError\":\"\"}",
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIP\":\"1.2.3.4\",\"targetPort\":\"443\",\"sourceIP\":\"5.6.7.8\",\"sourcePort\":\"8080\",\"isWarmup\":\"false\",\"roundTripTimeInMs\":20.00,\"isTimedOut\":\"false\",\"preparationError\":\"\",\"pingError\":\"\",\"handshakeError\":\"connect aborted\"}",
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIP\":\"1.2.3.4\",\"targetPort\":\"443\",\"sourceIP\":\"5.6.7.8\",\"sourcePort\":\"8080\",\"isWarmup\":\"false\",\"roundTripTimeInMs\":0.00,\"isTimedOut\":\"false\",\"preparationError\":\"\",\"pingError\":\"connect failed\",\"handshakeError\":\"\"}",
                "{\"utcTime\":\"2021-07-06T09:10:11.012Z\",\"protocol\":\"TCP\",\"workerId\":1,\"targetIP\":\"1.2.3.4\",\"targetPort\":\"443\",\"sourceIP\":\"5.6.7.8\",\"sourcePort\":\"8080\",\"isWarmup\":\"false\",\"roundTripTimeInMs\":0.00,\"isTimedOut\":\"false\",\"preparationError\":\"address in use\",\"pingError\":\"\",\"handshakeError\":\"\"}",
            ],
            results.into_iter().map(|x| x.format_as_json_string()).collect::<Vec<String>>()
        );
    }

    #[test]
    fn format_ping_result_as_csv_should_work() {
        let results = rnp_test_utils::generate_ping_result_test_samples();
        assert_eq!(
            vec![
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,true,10.00,false,\"\",\"\",\"\"",
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,false,1000.00,true,\"\",\"\",\"\"",
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,false,20.00,false,\"\",\"\",\"connect aborted\"",
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,false,0.00,false,\"\",\"connect failed\",\"\"",
                "2021-07-06T09:10:11.012Z,1,TCP,1.2.3.4,443,5.6.7.8,8080,false,0.00,false,\"address in use\",\"\",\"\"",
            ],
            results
                .into_iter()
                .map(|x| x.format_as_csv_string())
                .collect::<Vec<String>>()
        );
    }
}
