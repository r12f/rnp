use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::{rnp_utils, PingResult};
use std::{fs::File, io::prelude::*, path::PathBuf, io};
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
