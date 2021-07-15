use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::{rnp_utils, PingResult};
use std::{fs::File, io, io::prelude::*, path::PathBuf};
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
    fn initialize(&mut self) {
        // Writer json start
        self.log_file.write("[".as_bytes()).expect(&format!(
            "Failed to write logs to json file! Path = {}",
            self.log_path.display()
        ));
    }

    fn process_ping_result(&mut self, ping_result: &PingResult) {
        self.log_result_as_json(ping_result).expect(&format!(
            "Failed to write logs to json file! Path = {}",
            self.log_path.display()
        ));
    }

    fn rundown(&mut self) {
        // Writer json end
        self.log_file.write("\n]\n".as_bytes()).expect(&format!(
            "Failed to write logs to json file! Path = {}",
            self.log_path.display()
        ));
    }
}
