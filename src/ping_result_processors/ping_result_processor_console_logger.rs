use crate::*;
use std::io::{stdout, Write};
use std::net::SocketAddr;
use std::time::Instant;
use tracing;
use std::sync::Arc;

pub struct PingResultProcessorConsoleLogger {
    common_config: Arc<PingResultProcessorCommonConfig>,
    last_console_flush_time: Instant,

    protocol: Option<String>,
    target: Option<SocketAddr>,
    ping_count: u32,
    success_count: u32,
    failure_count: u32,
    handshake_failed_count: u32,
    disconnect_failed_count: u32,
    min_latency_in_us: u128,
    max_latency_in_us: u128,
    average_latency_in_us: f64,
}

impl PingResultProcessorConsoleLogger {
    #[tracing::instrument(name = "Creating ping result console logger", level = "debug")]
    pub fn new(common_config: Arc<PingResultProcessorCommonConfig>) -> PingResultProcessorConsoleLogger {
        return PingResultProcessorConsoleLogger {
            common_config,
            last_console_flush_time: Instant::now(),
            protocol: None,
            target: None,
            ping_count: 0,
            success_count: 0,
            failure_count: 0,
            handshake_failed_count: 0,
            disconnect_failed_count: 0,
            min_latency_in_us: u128::MAX,
            max_latency_in_us: u128::MIN,
            average_latency_in_us: 0.0,
        };
    }

    fn update_statistics(&mut self, ping_result: &PingResult) {
        // Skip warmup pings in analysis.
        if ping_result.is_warmup() {
            return;
        }

        // Skip preparation errors in analysis, since it is not a remote issue.
        if ping_result.is_preparation_error() {
            return;
        }

        // Save some info for outputting summary.
        if self.target.is_none() {
            self.protocol = Some(ping_result.protocol().to_string());
            self.target = Some(ping_result.target());
        }

        self.ping_count += 1;
        match ping_result.error() {
            Some(_) => self.failure_count += 1,
            None => self.success_count += 1,
        }

        if let Some(warning) = ping_result.warning() {
            match warning {
                PingClientWarning::AppHandshakeFailed(_) => self.handshake_failed_count += 1,
                PingClientWarning::DisconnectFailed(_) => self.disconnect_failed_count += 1,
            }
        };

        let latency_in_us = ping_result.round_trip_time().as_micros();
        if latency_in_us == 0 {
            // Latency data not set.
            return;
        }

        self.min_latency_in_us = std::cmp::min(latency_in_us, self.min_latency_in_us);
        self.max_latency_in_us = std::cmp::max(latency_in_us, self.max_latency_in_us);

        // Calculate moving average. Ping count already added 1 above.
        self.average_latency_in_us +=
            (latency_in_us as f64 - self.average_latency_in_us) / (self.ping_count as f64);
    }

    fn output_result_to_console(&mut self, ping_result: &PingResult) {
        if self.config().quiet_level == RNP_QUIET_LEVEL_REDUCE_PING_RESULT_OUTPUT {
            self.output_ping_count_update_to_console(false);
            return;
        }

        println!("{}", ping_result.format_as_console_log());
    }

    fn output_ping_count_update_to_console(&mut self, force: bool) {
        // Only flush once per sec at maximum to avoid frequent flushing.
        let now = Instant::now();
        let time_since_last_flush = now - self.last_console_flush_time;
        if !force {
            if time_since_last_flush.as_millis() < 1000 {
                return;
            }
        }

        self.last_console_flush_time = now;

        print!("\r{0} pings finished.", self.ping_count);

        // Console buffer flushes whenever it sees line breaks or buffer full, so we need
        // to force flush stdout to make the line taking effect.
        let mut stdout = stdout();
        stdout.flush().unwrap();
    }
}

impl PingResultProcessor for PingResultProcessorConsoleLogger {
    fn name(&self) -> &'static str {
        "ConsoleLogger"
    }

    fn config(&self) -> &PingResultProcessorCommonConfig { self.common_config.as_ref() }

    fn process_ping_result(&mut self, ping_result: &PingResult) {
        self.update_statistics(ping_result);
        self.output_result_to_console(ping_result);
    }

    fn rundown(&mut self) {
        if self.config().quiet_level == RNP_QUIET_LEVEL_REDUCE_PING_RESULT_OUTPUT {
            self.output_ping_count_update_to_console(true);
            println!();
        }

        // Didn't received any result, skip output statistics.
        if self.target.is_none() {
            return;
        }

        println!(
            "\n=== Connect statistics for {} {:?} ===",
            self.protocol.as_ref().unwrap(),
            self.target.as_ref().unwrap(),
        );

        let mut warning: String = String::from("");
        if self.handshake_failed_count > 0 || self.disconnect_failed_count > 0 {
            let mut warning_messages = Vec::new();
            if self.handshake_failed_count > 0 {
                warning_messages.push(format!(
                    "App Handshake Failed = {}",
                    self.handshake_failed_count
                ));
            }
            if self.disconnect_failed_count > 0 {
                warning_messages.push(format!(
                    "Disconnect Failed = {}",
                    self.disconnect_failed_count
                ));
            }
            warning = format!(" ({})", warning_messages.join(","));
        }

        println!(
            "- Connects: Sent = {}, Succeeded = {}{}, Failed = {} ({:.2}%).",
            self.ping_count,
            self.success_count,
            warning,
            self.failure_count,
            (self.failure_count as f64 * 100.0) / (self.ping_count as f64),
        );

        // If we haven't received any data, the min/max/average data won't be updated correctly,
        // os we output the data differently.
        if self.min_latency_in_us == u128::MAX {
            println!("- Round trip time: Minimum = 0ms, Maximum = 0ms, Average = 0ms.");
        } else {
            println!(
                "- Round trip time: Minimum = {:.2}ms, Maximum = {:.2}ms, Average = {:.2}ms.",
                self.min_latency_in_us as f64 / 1000.0,
                self.max_latency_in_us as f64 / 1000.0,
                self.average_latency_in_us / 1000.0
            );
        }
    }
}
