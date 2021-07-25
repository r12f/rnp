use crate::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use tracing;

pub struct PingResultProcessorTextLogger {
    log_path: PathBuf,
    log_file: File,
}

impl PingResultProcessorTextLogger {
    #[tracing::instrument(name = "Creating ping result text logger", level = "debug")]
    pub fn new(log_path_buf: &PathBuf) -> PingResultProcessorTextLogger {
        return PingResultProcessorTextLogger {
            log_path: log_path_buf.clone(),
            log_file: rnp_utils::create_log_file(log_path_buf),
        };
    }
}

impl PingResultProcessor for PingResultProcessorTextLogger {
    fn name(&self) -> &'static str {
        "TextLogger"
    }

    fn process_ping_result(&mut self, ping_result: &PingResult) {
        let log_content: String = ping_result.format_as_console_log();
        self.log_file.write(log_content.as_bytes()).expect(&format!(
            "Failed to write logs to text file! Path = {}",
            self.log_path.display()
        ));
        self.log_file.write("\n".as_bytes()).expect(&format!(
            "Failed to write logs to text file! Path = {}",
            self.log_path.display()
        ));
    }
}
