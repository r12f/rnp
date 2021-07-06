use rand::Rng;
use rnp::{
    PingClientConfig, PingResultProcessorConfig, PingWorkerConfig, PingWorkerSchedulerConfig,
    RnpCoreConfig,
};
use socket2::Protocol;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt, PartialOrd, PartialEq)]
#[structopt(name = rnp::RNP_NAME, author = rnp::RNP_AUTHOR, about = rnp::RNP_ABOUT)]
pub struct RnpCliOptions {
    pub target: SocketAddr,

    #[structopt(
        short = "s",
        long = "src-ip",
        default_value = "0.0.0.0",
        help = "Source IP address."
    )]
    pub source_ip: IpAddr,

    #[structopt(long = "src-port-min", help = "First source port to use in ping.")]
    pub source_port_min: Option<u16>,

    #[structopt(long = "src-port-max", help = "Last source port to use in ping.")]
    pub source_port_max: Option<u16>,

    #[structopt(long = "src-port", use_delimiter = true, help = "Source port list to use in ping.")]
    pub source_port_list: Option<Vec<u16>>,

    #[structopt(short = "n", long = "count", default_value = "4", help = "Ping count.")]
    pub ping_count: u32,

    #[structopt(short = "t", help = "Ping until stopped.")]
    pub ping_until_stopped: bool,

    #[structopt(
        short = "w",
        long = "timeout",
        default_value = "1000",
        help = "Wait time for each ping in milliseconds."
    )]
    pub wait_timeout_in_ms: u32,

    #[structopt(
        short = "i",
        long = "interval",
        default_value = "1000",
        help = "Sleep between each ping in milliseconds."
    )]
    pub ping_interval_in_ms: u32,

    #[structopt(long = "ttl", help = "Time to live.")]
    pub time_to_live: Option<u32>,

    #[structopt(
        short = "p",
        long = "parallel",
        default_value = "1",
        help = "Count of pings running in parallel."
    )]
    pub parallel_ping_count: u32,

    #[structopt(
        short = "q",
        long,
        help = "Don't log each ping result to console. Summary and other things will still be written to console."
    )]
    pub no_console_log: bool,

    #[structopt(
        long = "log-csv",
        parse(from_os_str),
        help = "Log ping results a csv file."
    )]
    pub csv_log_path: Option<PathBuf>,

    #[structopt(
        long = "log-json",
        parse(from_os_str),
        help = "Log ping results to a json file."
    )]
    pub json_log_path: Option<PathBuf>,

    #[structopt(
        long = "log-text",
        parse(from_os_str),
        help = "Log ping results to a text file."
    )]
    pub text_log_path: Option<PathBuf>,

    #[structopt(
        short = "r",
        long,
        help = "Show ping result scatter map after ping is done."
    )]
    pub show_result_scatter: bool,

    #[structopt(
        short = "l",
        long,
        help = "Show latency scatter map after ping is done."
    )]
    pub show_latency_scatter: bool,

    // TODO: Need some more thoughts. This is too naive.
    #[structopt(
        short = "h",
        long,
        help = "Show bucketed latency hit count after ping is done."
    )]
    pub show_latency_heatmap: bool,

    #[structopt(
        long = "latency-buckets",
        default_value = "10",
        help = "Set the number of buckets to use for bucketing latency."
    )]
    pub latency_heatmap_bucket_count: usize,
}

impl RnpCliOptions {
    pub fn prepare_to_use(&mut self) {
        if self.source_port_min.is_none() {
            self.source_port_min = Some(rand::thread_rng().gen_range(1024..20000));
        }

        if self.source_port_max.is_none() {
            self.source_port_max = Some(self.source_port_min.unwrap() + 10000);
        }

        if self.source_port_min > self.source_port_max {
            tracing::warn!("Min source port is larger than max, swapping to fix.");
            std::mem::swap(&mut self.source_port_min, &mut self.source_port_max);
        }

        if !self.ping_until_stopped && self.ping_count < 1 {
            tracing::warn!("Ping count cannot be less than 1, setting to 1 as minimum.");
            self.ping_count = 1;
        }

        if self.parallel_ping_count < 1 {
            tracing::warn!("Parallel ping count cannot be 0. Setting to 1 as minimum.");
            self.parallel_ping_count = 1;
        }
    }

    pub fn to_rnp_core_config(&self) -> RnpCoreConfig {
        let mut config = RnpCoreConfig {
            worker_config: PingWorkerConfig {
                protocol: Protocol::TCP,
                target: self.target,
                source_ip: self.source_ip,
                ping_interval: Duration::from_millis(self.ping_interval_in_ms.into()),
                ping_client_config: PingClientConfig {
                    wait_timeout: Duration::from_millis(self.wait_timeout_in_ms.into()),
                    time_to_live: self.time_to_live,
                },
            },
            worker_scheduler_config: PingWorkerSchedulerConfig {
                source_port_min: self.source_port_min.unwrap(),
                source_port_max: self.source_port_max.unwrap(),
                source_port_list: match &self.source_port_list {
                    Some(port_list) => Some(port_list.clone()),
                    None => None,
                },
                ping_count: None,
                parallel_ping_count: self.parallel_ping_count,
            },
            result_processor_config: PingResultProcessorConfig {
                no_console_log: self.no_console_log,
                csv_log_path: self.csv_log_path.clone(),
                json_log_path: self.json_log_path.clone(),
                text_log_path: self.text_log_path.clone(),
                show_result_scatter: self.show_result_scatter,
                show_latency_scatter: self.show_latency_scatter,
                latency_heatmap_bucket_count: if self.show_latency_heatmap {
                    Some(self.latency_heatmap_bucket_count)
                } else {
                    None
                },
            },
        };

        if !self.ping_until_stopped {
            config.worker_scheduler_config.ping_count = Some(self.ping_count);
        }

        return config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rnp::{
        PingClientConfig, PingResultProcessorConfig, PingWorkerConfig, PingWorkerSchedulerConfig,
        RnpCoreConfig,
    };
    use socket2::Protocol;
    use std::path::PathBuf;
    use std::time::Duration;
    use structopt::StructOpt;

    #[test]
    fn parsing_default_options_should_work() {
        assert_eq!(
            RnpCliOptions {
                target: "10.0.0.1:443".parse().unwrap(),
                source_ip: "0.0.0.0".parse().unwrap(),
                source_port_min: None,
                source_port_max: None,
                source_port_list: None,
                ping_count: 4,
                ping_until_stopped: false,
                wait_timeout_in_ms: 1000,
                ping_interval_in_ms: 1000,
                time_to_live: None,
                parallel_ping_count: 1,
                no_console_log: false,
                csv_log_path: None,
                json_log_path: None,
                text_log_path: None,
                show_result_scatter: false,
                show_latency_scatter: false,
                show_latency_heatmap: false,
                latency_heatmap_bucket_count: 10,
            },
            RnpCliOptions::from_iter(&["tp.exe", "10.0.0.1:443"])
        );
    }

    #[test]
    fn parsing_short_options_should_work() {
        assert_eq!(
            RnpCliOptions {
                target: "10.0.0.1:443".parse().unwrap(),
                source_ip: "10.0.0.2".parse().unwrap(),
                source_port_min: None,
                source_port_max: None,
                source_port_list: None,
                ping_count: 10,
                ping_until_stopped: true,
                wait_timeout_in_ms: 2000,
                ping_interval_in_ms: 1500,
                time_to_live: None,
                parallel_ping_count: 10,
                no_console_log: true,
                csv_log_path: None,
                json_log_path: None,
                text_log_path: None,
                show_result_scatter: true,
                show_latency_scatter: true,
                show_latency_heatmap: true,
                latency_heatmap_bucket_count: 10,
            },
            RnpCliOptions::from_iter(&[
                "tp.exe",
                "10.0.0.1:443",
                "-s",
                "10.0.0.2",
                "-n",
                "10",
                "-t",
                "-w",
                "2000",
                "-i",
                "1500",
                "-p",
                "10",
                "-q",
                "-r",
                "-l",
                "-h",
            ])
        );
    }

    #[test]
    fn parsing_long_options_should_work() {
        assert_eq!(
            RnpCliOptions {
                target: "10.0.0.1:443".parse().unwrap(),
                source_ip: "10.0.0.2".parse().unwrap(),
                source_port_min: Some(1024),
                source_port_max: Some(2048),
                source_port_list: Some(vec![1024, 1025, 1026]),
                ping_count: 10,
                ping_until_stopped: false,
                wait_timeout_in_ms: 2000,
                ping_interval_in_ms: 1500,
                time_to_live: Some(128),
                parallel_ping_count: 10,
                no_console_log: true,
                csv_log_path: Some(PathBuf::from("log.csv")),
                json_log_path: Some(PathBuf::from("log.json")),
                text_log_path: Some(PathBuf::from("log.txt")),
                show_result_scatter: true,
                show_latency_scatter: true,
                show_latency_heatmap: true,
                latency_heatmap_bucket_count: 20,
            },
            RnpCliOptions::from_iter(&[
                "tp.exe",
                "10.0.0.1:443",
                "--src-ip",
                "10.0.0.2",
                "--src-port-min",
                "1024",
                "--src-port-max",
                "2048",
                "--src-port",
                "1024,1025,1026",
                "--count",
                "10",
                "--timeout",
                "2000",
                "--interval",
                "1500",
                "--ttl",
                "128",
                "--parallel",
                "10",
                "--no-console-log",
                "--log-csv",
                "log.csv",
                "--log-json",
                "log.json",
                "--log-text",
                "log.txt",
                "--show-result-scatter",
                "--show-latency-scatter",
                "--show-latency-heatmap",
                "--latency-buckets",
                "20",
            ])
        );
    }

    #[test]
    fn new_from_cli_options_should_work() {
        assert_eq!(
            RnpCoreConfig {
                worker_config: PingWorkerConfig {
                    protocol: Protocol::TCP,
                    target: "10.0.0.1:443".parse().unwrap(),
                    source_ip: "10.0.0.2".parse().unwrap(),
                    ping_interval: Duration::from_millis(1500),
                    ping_client_config: PingClientConfig {
                        wait_timeout: Duration::from_millis(2000),
                        time_to_live: Some(128),
                    },
                },
                worker_scheduler_config: PingWorkerSchedulerConfig {
                    source_port_min: 1024,
                    source_port_max: 2047,
                    source_port_list: Some(vec![1024, 1025, 1026]),
                    ping_count: Some(4),
                    parallel_ping_count: 1,
                },
                result_processor_config: PingResultProcessorConfig {
                    no_console_log: false,
                    csv_log_path: None,
                    json_log_path: None,
                    text_log_path: None,
                    show_result_scatter: false,
                    show_latency_scatter: false,
                    latency_heatmap_bucket_count: None,
                },
            },
            RnpCliOptions {
                target: "10.0.0.1:443".parse().unwrap(),
                ping_count: 4,
                ping_until_stopped: false,
                source_ip: "10.0.0.2".parse().unwrap(),
                source_port_min: Some(1024),
                source_port_max: Some(2047),
                source_port_list: Some(vec![1024, 1025, 1026]),
                wait_timeout_in_ms: 2000,
                ping_interval_in_ms: 1500,
                time_to_live: Some(128),
                parallel_ping_count: 1,
                no_console_log: false,
                csv_log_path: None,
                json_log_path: None,
                text_log_path: None,
                show_result_scatter: false,
                show_latency_scatter: false,
                show_latency_heatmap: false,
                latency_heatmap_bucket_count: 10,
            }
            .to_rnp_core_config()
        );

        assert_eq!(
            RnpCoreConfig {
                worker_config: PingWorkerConfig {
                    protocol: Protocol::TCP,
                    target: "10.0.0.1:443".parse().unwrap(),
                    source_ip: "10.0.0.2".parse().unwrap(),
                    ping_interval: Duration::from_millis(1500),
                    ping_client_config: PingClientConfig {
                        wait_timeout: Duration::from_millis(2000),
                        time_to_live: Some(128),
                    },
                },
                worker_scheduler_config: PingWorkerSchedulerConfig {
                    source_port_min: 1024,
                    source_port_max: 2047,
                    source_port_list: Some(vec![1024, 1025, 1026]),
                    ping_count: None,
                    parallel_ping_count: 1,
                },
                result_processor_config: PingResultProcessorConfig {
                    no_console_log: true,
                    csv_log_path: Some(PathBuf::from("log.csv")),
                    json_log_path: Some(PathBuf::from("log.json")),
                    text_log_path: Some(PathBuf::from("log.txt")),
                    show_result_scatter: true,
                    show_latency_scatter: true,
                    latency_heatmap_bucket_count: Some(20),
                },
            },
            RnpCliOptions {
                target: "10.0.0.1:443".parse().unwrap(),
                ping_count: 4,
                ping_until_stopped: true,
                source_ip: "10.0.0.2".parse().unwrap(),
                source_port_min: Some(1024),
                source_port_max: Some(2047),
                source_port_list: Some(vec![1024, 1025, 1026]),
                wait_timeout_in_ms: 2000,
                ping_interval_in_ms: 1500,
                time_to_live: Some(128),
                parallel_ping_count: 1,
                no_console_log: true,
                csv_log_path: Some(PathBuf::from("log.csv")),
                json_log_path: Some(PathBuf::from("log.json")),
                text_log_path: Some(PathBuf::from("log.txt")),
                show_result_scatter: true,
                show_latency_scatter: true,
                show_latency_heatmap: true,
                latency_heatmap_bucket_count: 20,
            }
            .to_rnp_core_config()
        );
    }

    #[test]
    fn empty_source_port_in_options_should_be_fixed() {
        let mut opts = RnpCliOptions::from_iter(&["tp.exe", "10.0.0.1:443"]);
        opts.prepare_to_use();

        assert!(opts.source_port_min.is_some());
        assert!(opts.source_port_max.is_some());
        assert_eq!(
            10000,
            opts.source_port_max.unwrap() - opts.source_port_min.unwrap()
        );
    }

    #[test]
    fn invalid_options_should_be_fixed() {
        let mut opts = RnpCliOptions::from_iter(&["tp.exe", "10.0.0.1:443"]);
        opts.source_port_min = Some(2048);
        opts.source_port_max = Some(1024);
        opts.ping_count = 0;
        opts.parallel_ping_count = 0;
        opts.prepare_to_use();

        assert_eq!(1024, opts.source_port_min.unwrap());
        assert_eq!(2048, opts.source_port_max.unwrap());
        assert_eq!(1, opts.ping_count);
        assert_eq!(1, opts.parallel_ping_count);
    }
}
