use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::{rnp_utils, PingResult};
use std::{fs::File, io, io::prelude::*, path::PathBuf};
use tracing;

pub struct PingResultProcessorCsvLogger {
    log_path: PathBuf,
    log_file: File,
}

impl PingResultProcessorCsvLogger {
    #[tracing::instrument(name = "Creating ping result csv logger", level = "debug")]
    pub fn new(log_path_buf: &PathBuf) -> PingResultProcessorCsvLogger {
        return PingResultProcessorCsvLogger {
            log_path: log_path_buf.clone(),
            log_file: rnp_utils::create_log_file(log_path_buf),
        };
    }

    fn log_result_as_csv(&mut self, ping_result: &PingResult) -> io::Result<()> {
        let log_content = ping_result.format_as_csv_string();
        self.log_file.write(log_content.as_bytes())?;
        self.log_file.write("\n".as_bytes())?;
        return Ok(());
    }
}

impl PingResultProcessor for PingResultProcessorCsvLogger {
    fn initialize(&mut self) {
        // Writer CSV header
        self.log_file
            .write("UTCTime,WorkerId,Protocol,TargetIP,TargetPort,SourceIP,SourcePort,IsWarmup,IsSucceeded,RTTInMs,IsTimedOut,PreparationError,PingError,HandshakeError\n".as_bytes())
            .expect(&format!(
                "Failed to write logs to csv file! Path = {}",
                self.log_path.display()
            ));
    }

    fn process_ping_result(&mut self, ping_result: &PingResult) {
        self.log_result_as_csv(ping_result).expect(&format!(
            "Failed to write logs to csv file! Path = {}",
            self.log_path.display()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ping_result_processors::ping_result_processor_test_common;
    use crate::rnp_dto::PingResultCsvDto;
    use pretty_assertions::assert_eq;
    use chrono::{Utc, TimeZone};

    #[test]
    fn ping_result_process_csv_logger_should_work() {
        let test_log_file_path = "tests_data\\test_log.csv";
        let mut processor: Box<dyn PingResultProcessor + Send + Sync> = Box::new(PingResultProcessorCsvLogger::new(&PathBuf::from(test_log_file_path)));
        ping_result_processor_test_common::run_ping_result_processor_with_test_samples(
            &mut processor,
        );

        let mut actual_logged_records = Vec::new();
        {
            let mut csv_reader = csv::Reader::from_path(test_log_file_path).unwrap();
            for result in csv_reader.deserialize() {
                let actual_record: PingResultCsvDto = result.unwrap();
                actual_logged_records.push(actual_record);
            }
        }

        assert_eq!(
            vec![
                PingResultCsvDto {
                    utc_time: Utc.ymd(2021, 7, 6).and_hms_milli(9, 10, 11, 12),
                    worker_id: 1,
                    protocol: "TCP".to_string(),
                    target_ip: "1.2.3.4".parse().unwrap(),
                    target_port: 443,
                    source_ip: "5.6.7.8:8080".parse().unwrap(),
                    source_port: 8080,
                    is_warmup: true,
                    is_timed_out: true,
                    preparation_error: "".to_string(),
                    ping_error: "".to_string(),
                    rtt_in_ms: 10f64,
                    is_succeeded: false,
                    handshake_error: "".to_string()
                },
            ],
            actual_logged_records,
        );
    }
}
