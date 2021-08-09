use crate::ping_result_processors::ping_result_processor_console_logger::PingResultProcessorConsoleLogger;
use crate::ping_result_processors::ping_result_processor_csv_logger::PingResultProcessorCsvLogger;
use crate::ping_result_processors::ping_result_processor_json_logger::PingResultProcessorJsonLogger;
use crate::ping_result_processors::ping_result_processor_latency_bucket_logger::PingResultProcessorLatencyBucketLogger;
use crate::ping_result_processors::ping_result_processor_latency_scatter_logger::PingResultProcessorLatencyScatterLogger;
use crate::ping_result_processors::ping_result_processor_result_scatter_logger::PingResultProcessorResultScatterLogger;
use crate::ping_result_processors::ping_result_processor_text_logger::PingResultProcessorTextLogger;
use crate::{PingResultProcessor, PingResultProcessorConfig};
use std::sync::Arc;

pub fn new(
    config: &PingResultProcessorConfig,
    mut extra_ping_result_processors: Vec<Box<dyn PingResultProcessor + Send + Sync>>,
) -> Vec<Box<dyn PingResultProcessor + Send + Sync>> {
    let common_config = Arc::new(config.common_config.clone());
    let mut processors = Vec::new();

    // We always create the console logger for keeping our user informed.
    let console_logger: Box<dyn PingResultProcessor + Send + Sync> = Box::new(PingResultProcessorConsoleLogger::new(common_config.clone()));
    processors.push(console_logger);

    if let Some(csv_log_path) = &config.csv_log_path {
        let csv_logger: Box<dyn PingResultProcessor + Send + Sync> = Box::new(PingResultProcessorCsvLogger::new(common_config.clone(), csv_log_path));
        processors.push(csv_logger);
    }

    if let Some(json_log_path) = &config.json_log_path {
        let json_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorJsonLogger::new(common_config.clone(), json_log_path));
        processors.push(json_logger);
    }

    if let Some(text_log_path) = &config.text_log_path {
        let text_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorTextLogger::new(common_config.clone(), text_log_path));
        processors.push(text_logger);
    }

    if config.show_result_scatter {
        let result_scatter_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorResultScatterLogger::new(common_config.clone()));
        processors.push(result_scatter_logger);
    }

    if config.show_latency_scatter {
        let latency_scatter_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorLatencyScatterLogger::new(common_config.clone()));
        processors.push(latency_scatter_logger);
    }

    if let Some(latency_buckets) = &config.latency_buckets {
        let latency_bucket_logger: Box<dyn PingResultProcessor + Send + Sync> =
            Box::new(PingResultProcessorLatencyBucketLogger::new(common_config.clone(), latency_buckets));
        processors.push(latency_bucket_logger);
    }

    // Move all extra ping result processors into the processors
    processors.append(&mut extra_ping_result_processors);

    return processors;
}

#[cfg(test)]
mod tests {
    use crate::ping_result_processors::ping_result_processor_factory::new;
    use crate::*;
    use std::path::PathBuf;

    #[test]
    fn create_ping_result_processor_should_work_with_empty_config() {
        let config = PingResultProcessorConfig {
            common_config: PingResultProcessorCommonConfig { quiet_level: RNP_QUIET_LEVEL_NONE },
            csv_log_path: None,
            json_log_path: None,
            text_log_path: None,
            show_result_scatter: false,
            show_latency_scatter: false,
            latency_buckets: None,
        };

        let ping_clients = new(&config, vec![]);
        assert_eq!(1, ping_clients.len());
    }

    #[test]
    fn create_ping_result_processor_should_work_with_valid_config() {
        let config = PingResultProcessorConfig {
            common_config: PingResultProcessorCommonConfig { quiet_level: RNP_QUIET_LEVEL_NO_PING_RESULT },
            csv_log_path: Some(PathBuf::from("log.csv")),
            json_log_path: Some(PathBuf::from("log.json")),
            text_log_path: Some(PathBuf::from("log.txt")),
            show_result_scatter: true,
            show_latency_scatter: true,
            latency_buckets: Some(vec![0.1, 0.5, 1.0, 10.0]),
        };

        let ping_clients = new(&config, vec![]);
        assert_eq!(7, ping_clients.len());
    }
}
