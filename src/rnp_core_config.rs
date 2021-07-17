use std::net::{IpAddr, SocketAddr};
use std::{time::Duration, path::PathBuf};

pub const RNP_NAME: &str = "rnp";
pub const RNP_AUTHOR: &str = "r12f (r12f.com, github.com/r12f)";
pub const RNP_ABOUT: &str = "A simple layer 4 ping tool for cloud.";

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum RnpSupportedProtocol {
    TCP,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RnpCoreConfig {
    pub worker_config: PingWorkerConfig,
    pub worker_scheduler_config: PingWorkerSchedulerConfig,
    pub result_processor_config: PingResultProcessorConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingWorkerConfig {
    pub protocol: RnpSupportedProtocol,
    pub target: SocketAddr,
    pub source_ip: IpAddr,
    pub ping_interval: Duration,
    pub ping_client_config: PingClientConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingClientConfig {
    pub wait_timeout: Duration,
    pub time_to_live: Option<u32>,
    pub use_fin_in_tcp_ping: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingWorkerSchedulerConfig {
    pub source_port_min: u16,
    pub source_port_max: u16,
    pub source_port_list: Option<Vec<u16>>,
    pub ping_count: Option<u32>,
    pub warmup_count: u32,
    pub parallel_ping_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingResultProcessorConfig {
    pub no_console_log: bool,
    pub csv_log_path: Option<PathBuf>,
    pub json_log_path: Option<PathBuf>,
    pub text_log_path: Option<PathBuf>,
    pub show_result_scatter: bool,
    pub show_latency_scatter: bool,
    pub latency_buckets: Option<Vec<f64>>,
}
