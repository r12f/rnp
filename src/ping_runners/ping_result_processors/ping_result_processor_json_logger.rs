use crate::*;
use std::sync::Arc;
use std::{fs::File, io, io::prelude::*, path::PathBuf};
use tracing;

pub struct PingResultProcessorJsonLogger {
    common_config: Arc<PingResultProcessorCommonConfig>,
    log_path: PathBuf,
    log_file: File,
    is_first_element: bool,
}

impl PingResultProcessorJsonLogger {
    #[tracing::instrument(name = "Creating ping result json logger", level = "debug")]
    pub fn new(common_config: Arc<PingResultProcessorCommonConfig>, log_path_buf: &PathBuf) -> PingResultProcessorJsonLogger {
        return PingResultProcessorJsonLogger {
            common_config,
            log_path: log_path_buf.clone(),
            log_file: rnp_utils::create_log_file(log_path_buf),
            is_first_element: true,
        };
    }

    fn log_result_as_json(&mut self, ping_result: &PingResult) -> io::Result<()> {
        if self.is_first_element {
            self.is_first_element = false;
            self.log_file.write("\n  ".as_bytes())?;
        } else {
            self.log_file.write(",\n  ".as_bytes())?;
        }

        let log_content = ping_result.format_as_json_string();
        self.log_file.write(log_content.as_bytes())?;

        return Ok(());
    }
}

impl PingResultProcessor for PingResultProcessorJsonLogger {
    fn name(&self) -> &'static str {
        "JsonLogger"
    }
    fn config(&self) -> &PingResultProcessorCommonConfig {
        self.common_config.as_ref()
    }

    fn initialize(&mut self) {
        // Writer json start
        self.log_file.write("[".as_bytes()).expect(&format!("Failed to write logs to json file! Path = {}", self.log_path.display()));
    }

    fn process_ping_result(&mut self, ping_result: &PingResult) {
        self.log_result_as_json(ping_result).expect(&format!("Failed to write logs to json file! Path = {}", self.log_path.display()));
    }

    fn rundown(&mut self) {
        // Writer json end
        self.log_file.write("\n]\n".as_bytes()).expect(&format!("Failed to write logs to json file! Path = {}", self.log_path.display()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ping_result_processors::ping_result_processor_test_common;
    use crate::PingResultDto;
    use chrono::{TimeZone, Utc};
    use pretty_assertions::assert_eq;
    use std::io::BufReader;

    #[test]
    fn ping_result_process_json_logger_should_work() {
        let test_log_file_path = "tests_data/ping_result_processor_json_logger_tests/test_log.json";
        let mut processor: Box<dyn PingResultProcessor + Send + Sync> = Box::new(PingResultProcessorJsonLogger::new(
            Arc::new(PingResultProcessorCommonConfig { quiet_level: RNP_QUIET_LEVEL_NO_OUTPUT }),
            &PathBuf::from(test_log_file_path),
        ));
        ping_result_processor_test_common::run_ping_result_processor_with_test_samples(&mut processor);

        let actual_logged_records: Vec<PingResultDto>;
        {
            let test_log_file = File::open(test_log_file_path).unwrap();
            let test_log_reader = BufReader::new(test_log_file);
            actual_logged_records = serde_json::from_reader(test_log_reader).unwrap();
        }

        assert_eq!(
            vec![
                PingResultDto {
                    utc_time: Utc.with_ymd_and_hms(2021, 7, 6, 9, 10, 11).unwrap() + chrono::Duration::milliseconds(12),
                    worker_id: 1,
                    protocol: "TCP".to_string(),
                    target_ip: "1.2.3.4".parse().unwrap(),
                    target_port: 443,
                    source_ip: "5.6.7.8".parse().unwrap(),
                    source_port: 8080,
                    is_warmup: true,
                    is_succeeded: true,
                    is_timed_out: false,
                    rtt_in_ms: 10f64,
                    preparation_error: "".to_string(),
                    ping_error: "".to_string(),
                    handshake_error: "".to_string(),
                    disconnect_error: "".to_string(),
                },
                PingResultDto {
                    utc_time: Utc.with_ymd_and_hms(2021, 7, 6, 9, 10, 11).unwrap() + chrono::Duration::milliseconds(12),
                    worker_id: 1,
                    protocol: "TCP".to_string(),
                    target_ip: "1.2.3.4".parse().unwrap(),
                    target_port: 443,
                    source_ip: "5.6.7.8".parse().unwrap(),
                    source_port: 8080,
                    is_warmup: false,
                    is_succeeded: false,
                    is_timed_out: true,
                    rtt_in_ms: 1000f64,
                    preparation_error: "".to_string(),
                    ping_error: "".to_string(),
                    handshake_error: "".to_string(),
                    disconnect_error: "".to_string(),
                },
                PingResultDto {
                    utc_time: Utc.with_ymd_and_hms(2021, 7, 6, 9, 10, 11).unwrap() + chrono::Duration::milliseconds(12),
                    worker_id: 1,
                    protocol: "TCP".to_string(),
                    target_ip: "1.2.3.4".parse().unwrap(),
                    target_port: 443,
                    source_ip: "5.6.7.8".parse().unwrap(),
                    source_port: 8080,
                    is_warmup: false,
                    is_succeeded: true,
                    is_timed_out: false,
                    rtt_in_ms: 20f64,
                    preparation_error: "".to_string(),
                    ping_error: "".to_string(),
                    handshake_error: "connect aborted".to_string(),
                    disconnect_error: "".to_string(),
                },
                PingResultDto {
                    utc_time: Utc.with_ymd_and_hms(2021, 7, 6, 9, 10, 11).unwrap() + chrono::Duration::milliseconds(12),
                    worker_id: 1,
                    protocol: "TCP".to_string(),
                    target_ip: "1.2.3.4".parse().unwrap(),
                    target_port: 443,
                    source_ip: "5.6.7.8".parse().unwrap(),
                    source_port: 8080,
                    is_warmup: false,
                    is_succeeded: true,
                    is_timed_out: false,
                    rtt_in_ms: 20f64,
                    preparation_error: "".to_string(),
                    ping_error: "".to_string(),
                    handshake_error: "".to_string(),
                    disconnect_error: "disconnect timeout".to_string(),
                },
                PingResultDto {
                    utc_time: Utc.with_ymd_and_hms(2021, 7, 6, 9, 10, 11).unwrap() + chrono::Duration::milliseconds(12),
                    worker_id: 1,
                    protocol: "TCP".to_string(),
                    target_ip: "1.2.3.4".parse().unwrap(),
                    target_port: 443,
                    source_ip: "5.6.7.8".parse().unwrap(),
                    source_port: 8080,
                    is_warmup: false,
                    is_succeeded: false,
                    is_timed_out: false,
                    rtt_in_ms: 0f64,
                    preparation_error: "".to_string(),
                    ping_error: "connect failed".to_string(),
                    handshake_error: "".to_string(),
                    disconnect_error: "".to_string(),
                },
                PingResultDto {
                    utc_time: Utc.with_ymd_and_hms(2021, 7, 6, 9, 10, 11).unwrap() + chrono::Duration::milliseconds(12),
                    worker_id: 1,
                    protocol: "TCP".to_string(),
                    target_ip: "1.2.3.4".parse().unwrap(),
                    target_port: 443,
                    source_ip: "5.6.7.8".parse().unwrap(),
                    source_port: 8080,
                    is_warmup: false,
                    is_succeeded: false,
                    is_timed_out: false,
                    rtt_in_ms: 0f64,
                    preparation_error: "address in use".to_string(),
                    ping_error: "".to_string(),
                    handshake_error: "".to_string(),
                    disconnect_error: "".to_string(),
                },
            ],
            actual_logged_records,
        );
    }
}
