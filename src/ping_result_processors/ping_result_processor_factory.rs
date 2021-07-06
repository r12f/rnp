use crate::ping_result_processors::ping_result_processor_console_logger::PingResultProcessorConsoleLogger;
use crate::ping_result_processors::ping_result_processor_csv_logger::PingResultProcessorCsvLogger;
use crate::ping_result_processors::ping_result_processor_json_logger::PingResultProcessorJsonLogger;
use crate::ping_result_processors::ping_result_processor_latency_scatter_logger::PingResultProcessorLatencyScatterLogger;
use crate::ping_result_processors::ping_result_processor_result_scatter_logger::PingResultProcessorResultScatterLogger;
use crate::ping_result_processors::ping_result_processor_text_logger::PingResultProcessorTextLogger;
use crate::{PingResultProcessor, PingResultProcessorConfig};

pub fn new(config: &PingResultProcessorConfig) -> Vec<Box<dyn PingResultProcessor + Send + Sync>> {
    let mut processors = Vec::new();

    // We always create the console logger for keeping our user informed.
    let console_logger: Box<dyn PingResultProcessor + Send + Sync> =
        Box::new(PingResultProcessorConsoleLogger::new(config.no_console_log));
    processors.push(console_logger);

    if let Some(csv_log_path) = &config.csv_log_path {
        let csv_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorCsvLogger::new(csv_log_path));
        processors.push(csv_logger);
    }

    if let Some(json_log_path) = &config.json_log_path {
        let json_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorJsonLogger::new(json_log_path));
        processors.push(json_logger);
    }

    if let Some(text_log_path) = &config.text_log_path {
        let text_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorTextLogger::new(text_log_path));
        processors.push(text_logger);
    }

    if config.show_result_scatter {
        let result_scatter_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorResultScatterLogger::new());
        processors.push(result_scatter_logger);
    }

    if config.show_latency_scatter {
        let latency_scatter_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorLatencyScatterLogger::new());
        processors.push(latency_scatter_logger);
    }

    return processors;
}

#[cfg(test)]
mod tests {
    use crate::ping_result_processors::ping_result_processor_factory::new;
    use crate::PingResultProcessorConfig;
    use std::path::PathBuf;

    #[test]
    fn create_ping_result_processor_should_work_with_empty_config() {
        let config = PingResultProcessorConfig {
            no_console_log: false,
            csv_log_path: None,
            json_log_path: None,
            text_log_path: None,
            show_result_scatter: false,
            show_latency_scatter: false,
            latency_heatmap_bucket_count: None,
        };

        let ping_clients = new(&config);
        assert_eq!(1, ping_clients.len());
    }

    #[test]
    fn create_ping_result_processor_should_work_with_valid_config() {
        let config = PingResultProcessorConfig {
            no_console_log: true,
            csv_log_path: Some(PathBuf::from("log.csv")),
            json_log_path: Some(PathBuf::from("log.json")),
            text_log_path: Some(PathBuf::from("log.txt")),
            show_result_scatter: true,
            show_latency_scatter: true,
            latency_heatmap_bucket_count: Some(20),
        };

        let ping_clients = new(&config);
        assert_eq!(6, ping_clients.len());
    }
}
