use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::{rnp_utils, PingResult};
use std::{fs::File, io::prelude::*, path::PathBuf};
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
}

impl PingResultProcessor for PingResultProcessorCsvLogger {
    fn prepare(&mut self) {
        // Writer CSV header
        self.log_file
            .write("UTCTime,WorkerId,Protocol,Target,Source,RTTInMs,Error\n".as_bytes())
            .expect(&format!(
                "Failed to write logs to csv file! Path = {}",
                self.log_path.display()
            ));
    }

    fn process(&mut self, ping_result: &PingResult) {
        let mut error_message: String = String::from("");
        if let Some(e) = ping_result.error() {
            error_message = format!("{}", e);
        };

        let log_content = &format!(
            "{:?},{},{},{},{},{:.2},\"{}\"\n",
            ping_result.utc_time(),
            ping_result.protocol_string(),
            ping_result.worker_id(),
            ping_result.target(),
            ping_result.source(),
            ping_result.round_trip_time().as_micros() as f64 / 1000.0,
            error_message,
        );

        self.log_file.write(log_content.as_bytes()).expect(&format!(
            "Failed to write logs to csv file! Path = {}",
            self.log_path.display()
        ));
    }
}
