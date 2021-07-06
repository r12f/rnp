use socket2::Protocol;
use std::net::{IpAddr, SocketAddr};
use std::{time::Duration, path::PathBuf};

pub const RNP_NAME: &str = "rnp";
pub const RNP_AUTHOR: &str = "r12f (r12f.com, github.com/r12f)";
pub const RNP_ABOUT: &str = "A simple cloud-friendly tool for testing network reachability.";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RnpCoreConfig {
    pub worker_config: PingWorkerConfig,
    pub worker_scheduler_config: PingWorkerSchedulerConfig,
    pub result_processor_config: PingResultProcessorConfig,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PingWorkerConfig {
    pub protocol: Protocol,
    pub target: SocketAddr,
    pub source_ip: IpAddr,
    pub ping_interval: Duration,
    pub ping_client_config: PingClientConfig,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PingClientConfig {
    pub wait_timeout: Duration,
    pub time_to_live: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PingWorkerSchedulerConfig {
    pub source_port_min: u16,
    pub source_port_max: u16,
    pub ping_count: Option<u32>,
    pub parallel_ping_count: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PingResultProcessorConfig {
    pub no_console_log: bool,
    pub csv_log_path: Option<PathBuf>,
    pub json_log_path: Option<PathBuf>,
    pub text_log_path: Option<PathBuf>,
    pub show_result_scatter: bool,
    pub show_latency_scatter: bool,
    pub latency_heatmap_bucket_count: Option<usize>,
}
