use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::{rnp_utils, PingResult};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use tracing;

pub struct PingResultProcessorJsonLogger {
    log_path: PathBuf,
    log_file: File,
    is_first_element: bool,
}

impl PingResultProcessorJsonLogger {
    #[tracing::instrument(name = "Creating ping result json logger", level = "debug")]
    pub fn new(log_path_buf: &PathBuf) -> PingResultProcessorJsonLogger {
        return PingResultProcessorJsonLogger {
            log_path: log_path_buf.clone(),
            log_file: rnp_utils::create_log_file(log_path_buf),
            is_first_element: true,
        };
    }
}

impl PingResultProcessor for PingResultProcessorJsonLogger {
    fn prepare(&mut self) {
        // Writer json start
        self.log_file
            .write("[".as_bytes())
            .expect(&format!(
                "Failed to write logs to json file! Path = {}",
                self.log_path.display()
            ));
    }

    fn process(&mut self, ping_result: &PingResult) {
        let mut line_prefix = ",";
        if self.is_first_element {
            self.is_first_element = false;
            line_prefix = ""
        }

        let mut error_message: String = String::from("");
        if let Some(e) = ping_result.error() {
            error_message = format!("{}", e);
        };

        let log_content = &format!(
            "{}\n  {{\"utcTime\":\"{:?}\",\"protocol\":\"{}\",\"workerId\":{},\"target\":\"{}\",\"source\":\"{}\",\"roundTripTimeInMs\":{:.3},\"error\":\"{}\"}}",
            line_prefix,
            ping_result.utc_time(),
            ping_result.protocol_string(),
            ping_result.worker_id(),
            ping_result.target(),
            ping_result.source(),
            ping_result.round_trip_time().as_micros() as f64 / 1000.0,
            error_message,
        );

        self.log_file.write(log_content.as_bytes()).expect(&format!(
            "Failed to write logs to json file! Path = {}",
            self.log_path.display()
        ));
    }

    fn done(&mut self) {
        // Writer json end
        self.log_file
            .write("\n]\n".as_bytes())
            .expect(&format!(
                "Failed to write logs to json file! Path = {}",
                self.log_path.display()
            ));
    }
}
