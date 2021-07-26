use crate::*;
use chrono::{TimeZone, Utc};
use std::io;
use std::time::Duration;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        // initialization code here
        let _ = env_logger::builder().is_test(true).try_init();
    });
}

pub fn generate_ping_result_test_samples() -> Vec<PingResult> {
    vec![
        // Succeeded + Warmup
        PingResult::new(
            &Utc.ymd(2021, 7, 6).and_hms_milli(9, 10, 11, 12),
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
        ),
        // Timeout
        PingResult::new(
            &Utc.ymd(2021, 7, 6).and_hms_milli(9, 10, 11, 12),
            1,
            "TCP",
            "1.2.3.4:443".parse().unwrap(),
            "5.6.7.8:8080".parse().unwrap(),
            false,
            false,
            Duration::from_millis(1000),
            true,
            None,
            None,
        ),
        // Reachable but got handshake failure
        PingResult::new(
            &Utc.ymd(2021, 7, 6).and_hms_milli(9, 10, 11, 12),
            1,
            "TCP",
            "1.2.3.4:443".parse().unwrap(),
            "5.6.7.8:8080".parse().unwrap(),
            false,
            true,
            Duration::from_millis(20),
            false,
            Some(PingClientWarning::AppHandshakeFailed(Box::new(
                io::Error::new(io::ErrorKind::ConnectionAborted, "connect aborted"),
            ))),
            None,
        ),
        // Reachable but disconnect connection timed out
        PingResult::new(
            &Utc.ymd(2021, 7, 6).and_hms_milli(9, 10, 11, 12),
            1,
            "TCP",
            "1.2.3.4:443".parse().unwrap(),
            "5.6.7.8:8080".parse().unwrap(),
            false,
            true,
            Duration::from_millis(20),
            false,
            Some(PingClientWarning::DisconnectFailed(Box::new(
                io::Error::new(io::ErrorKind::TimedOut, "disconnect timeout"),
            ))),
            None,
        ),
        // Failed to reach remote
        PingResult::new(
            &Utc.ymd(2021, 7, 6).and_hms_milli(9, 10, 11, 12),
            1,
            "TCP",
            "1.2.3.4:443".parse().unwrap(),
            "5.6.7.8:8080".parse().unwrap(),
            false,
            false,
            Duration::from_millis(0),
            false,
            None,
            Some(PingClientError::PingFailed(Box::new(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "connect failed",
            )))),
        ),
        // Failed to create local resources for ping, such as cannot bind address
        PingResult::new(
            &Utc.ymd(2021, 7, 6).and_hms_milli(9, 10, 11, 12),
            1,
            "TCP",
            "1.2.3.4:443".parse().unwrap(),
            "5.6.7.8:8080".parse().unwrap(),
            false,
            false,
            Duration::from_millis(0),
            false,
            None,
            Some(PingClientError::PreparationFailed(Box::new(io::Error::new(
                io::ErrorKind::AddrInUse,
                "address in use",
            )))),
        ),
    ]
}
