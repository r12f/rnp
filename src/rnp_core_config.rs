use crate::{PingClientFactory, PingResultProcessor, PortRangeList};
use std::fmt;
use std::fmt::Debug;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::{path::PathBuf, time::Duration};

pub const RNP_NAME: &str = "rnp";
pub const RNP_AUTHOR: &str = "r12f (r12f.com, github.com/r12f)";
pub const RNP_ABOUT: &str = "A simple layer 4 ping tool for cloud.";

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum RnpSupportedProtocol {
    TCP,
    QUIC,
    External(String),
}

impl FromStr for RnpSupportedProtocol {
    type Err = String;

    fn from_str(input: &str) -> Result<RnpSupportedProtocol, Self::Err> {
        match input.to_uppercase().as_str() {
            "TCP" => Ok(RnpSupportedProtocol::TCP),
            "QUIC" => Ok(RnpSupportedProtocol::QUIC),
            _ => Err(String::from("Invalid protocol")),
        }
    }
}

impl fmt::Display for RnpSupportedProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let protocol = match self {
            RnpSupportedProtocol::TCP => "TCP",
            RnpSupportedProtocol::QUIC => "QUIC",
            RnpSupportedProtocol::External(p) => &p,
        };

        write!(f, "{}", protocol)
    }
}

pub struct RnpCoreConfig {
    pub worker_config: PingWorkerConfig,
    pub worker_scheduler_config: PingWorkerSchedulerConfig,
    pub result_processor_config: PingResultProcessorConfig,
    pub external_ping_client_factory: Option<PingClientFactory>,
    pub extra_ping_result_processors: Vec<Box<dyn PingResultProcessor + Send + Sync>>,
}

impl Debug for RnpCoreConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RnpCoreConfig")
            .field("worker_config", &self.worker_config)
            .field("worker_scheduler_config", &self.worker_scheduler_config)
            .field("result_processor_config", &self.result_processor_config)
            .field(
                "external_ping_client_factory",
                &if self.external_ping_client_factory.is_some() { "Some(PingClientFactory)".to_string() } else { "None".to_string() },
            )
            .field("extra_ping_result_processors", &self.extra_ping_result_processors.iter().map(|p| p.name()).collect::<Vec<&'static str>>())
            .finish()
    }
}

impl PartialEq for RnpCoreConfig {
    fn eq(&self, other: &RnpCoreConfig) -> bool {
        if self.worker_config != other.worker_config {
            return false;
        }
        if self.worker_scheduler_config != other.worker_scheduler_config {
            return false;
        }
        if self.result_processor_config != other.result_processor_config {
            return false;
        }
        if self.external_ping_client_factory.is_some() != other.external_ping_client_factory.is_some() {
            return false;
        }
        let matching_processor_count =
            self.extra_ping_result_processors.iter().zip(other.extra_ping_result_processors.iter()).filter(|&(a, b)| a.name() == b.name()).count();
        return matching_processor_count == self.extra_ping_result_processors.len()
            && matching_processor_count == other.extra_ping_result_processors.len();
    }
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
    pub check_disconnect: bool,
    pub server_name: Option<String>,
    pub log_tls_key: bool,
    pub alpn_protocol: Option<String>,
    pub use_timer_rtt: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingWorkerSchedulerConfig {
    pub source_ports: PortRangeList,
    pub ping_count: Option<u32>,
    pub warmup_count: u32,
    pub parallel_ping_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingResultProcessorCommonConfig {
    pub quiet_level: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingResultProcessorConfig {
    pub common_config: PingResultProcessorCommonConfig,
    pub csv_log_path: Option<PathBuf>,
    pub json_log_path: Option<PathBuf>,
    pub text_log_path: Option<PathBuf>,
    pub show_result_scatter: bool,
    pub show_latency_scatter: bool,
    pub latency_buckets: Option<Vec<f64>>,
}

pub const RNP_QUIET_LEVEL_NONE: i32 = 0;
pub const RNP_QUIET_LEVEL_NO_PING_RESULT: i32 = 1;
pub const RNP_QUIET_LEVEL_NO_PING_SUMMARY: i32 = 2;
pub const RNP_QUIET_LEVEL_NO_OUTPUT: i32 = 3;
