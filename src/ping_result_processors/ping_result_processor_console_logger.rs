use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::PingResult;
use std::io::{stdout, Write};
use std::net::SocketAddr;
use std::time::Instant;
use tracing;

pub struct PingResultProcessorConsoleLogger {
    no_console_log: bool,
    last_console_flush_time: Instant,

    target: Option<SocketAddr>,
    ping_count: u32,
    success_count: u32,
    failure_count: u32,
    min_latency_in_us: u128,
    max_latency_in_us: u128,
    average_latency_in_us: f64,
}

impl PingResultProcessorConsoleLogger {
    #[tracing::instrument(name = "Creating ping result console logger", level = "debug")]
    pub fn new(no_console_log: bool) -> PingResultProcessorConsoleLogger {
        return PingResultProcessorConsoleLogger {
            no_console_log,
            last_console_flush_time: Instant::now(),
            target: None,
            ping_count: 0,
            success_count: 0,
            failure_count: 0,
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

        // Save the target for outputting summary.
        if self.target.is_none() {
            self.target = Some(ping_result.target());
        }

        self.ping_count += 1;
        match ping_result.error() {
            Some(_) => self.failure_count += 1,
            None => self.success_count += 1,
        }

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
        if self.no_console_log {
            self.output_ping_count_update_to_console();
            return;
        }

        println!("{}", ping_result.format_as_console_log());
    }

    fn output_ping_count_update_to_console(&mut self) {
        // Only flush once per sec at maximum to avoid frequent flushing.
        let now = Instant::now();
        let time_since_last_flush = now - self.last_console_flush_time;
        if time_since_last_flush.as_millis() < 1000 {
            return;
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
    fn process_ping_result(&mut self, ping_result: &PingResult) {
        self.update_statistics(ping_result);
        self.output_result_to_console(ping_result);
    }

    fn rundown(&mut self) {
        // Didn't received any result, skip output statistics.
        if self.target.is_none() {
            return;
        }

        println!(
            "\n=== TCP connect statistics for {:?} ===",
            self.target.unwrap()
        );

        println!(
            "- Packets: Sent = {}, Received = {}, Lost = {} ({:.2}% loss).",
            self.ping_count,
            self.success_count,
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
