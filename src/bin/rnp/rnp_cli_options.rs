use rand::Rng;
use rnp::{
    parse_ping_target, PingClientConfig, PingResultProcessorCommonConfig, PingResultProcessorConfig, PingWorkerConfig, PingWorkerSchedulerConfig,
    PortRangeList, RnpPingRunnerConfig, RnpSupportedProtocol,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt, PartialEq)]
#[structopt(name = rnp::RNP_NAME, author = rnp::RNP_AUTHOR, about = rnp::RNP_ABOUT)]
pub struct RnpCliOptions {
    #[structopt(flatten)]
    pub common_options: RnpCliCommonOptions,

    #[structopt(flatten)]
    pub ping_common_options: RnpCliPingCommonOptions,

    #[structopt(flatten)]
    pub output_options: RnpCliOutputOptions,

    #[structopt(flatten)]
    pub quic_options: RnpCliQuicPingOptions,
}

#[derive(Debug, StructOpt, PartialEq)]
pub struct RnpCliCommonOptions {
    #[structopt(short = "m", long = "mode", default_value = "TCP", help = "Specify protocol to use.")]
    pub protocol: RnpSupportedProtocol,

    #[structopt(parse(try_from_str = parse_ping_target), help = "Target endpoint. For IPv6, please use [] to wrap the address, such as [::1]:80.")]
    pub target: SocketAddr,
}

#[derive(Debug, StructOpt, PartialEq)]
pub struct RnpCliPingCommonOptions {
    #[structopt(short = "s", long = "src-ip", default_value = "0.0.0.0", help = "Source IP address.")]
    pub source_ip: IpAddr,

    #[structopt(
        long = "src-ports",
        alias = "sp",
        help = "Source port ranges to rotate in ping. Format: port,start-end. Example: 1024,10000-11000. [alias: --sp]"
    )]
    pub source_ports: Option<PortRangeList>,

    #[structopt(short = "n", long = "count", default_value = "4", help = "Ping count.")]
    pub ping_count: u32,

    #[structopt(short = "t", help = "Ping until stopped.")]
    pub ping_until_stopped: bool,

    #[structopt(long = "warmup", default_value = "0", help = "Warm up ping count.")]
    pub warmup_count: u32,

    #[structopt(short = "w", long = "timeout", default_value = "2000", help = "Wait time for each ping in milliseconds.")]
    pub wait_timeout_in_ms: u32,

    #[structopt(short = "i", long = "interval", default_value = "1000", help = "Sleep between each ping in milliseconds.")]
    pub ping_interval_in_ms: u32,

    #[structopt(long = "ttl", help = "Time to live.")]
    pub time_to_live: Option<u32>,

    #[structopt(
        short = "d",
        long = "check-disconnect",
        help = "Check if connection can be correctly disconnected. Only available in TCP mode now.\nWhen enabled, we will use normal disconnect (w/ FIN) and check the connection disconnect."
    )]
    pub check_disconnect: bool,

    #[structopt(
        long = "wait-before-disconnect",
        default_value = "0",
        help = "Wait before disconnect in milliseconds. Only works when check-disconnect is enabled."
    )]
    pub wait_before_disconnect_in_ms: u64,

    #[structopt(short = "p", long = "parallel", default_value = "1", help = "Count of pings running in parallel.")]
    pub parallel_ping_count: u32,

    #[structopt(long, help = "Exit as soon as a ping failed and return a non-zero error code.")]
    pub exit_on_fail: bool,
}

#[derive(Debug, StructOpt, PartialEq)]
pub struct RnpCliOutputOptions {
    #[structopt(
        short = "q",
        parse(from_occurrences),
        help = "Quiet mode. -q = Don't output each ping result; -qq = Don't output final summaries; -qqq = Don't output anything."
    )]
    pub quiet_level: i32,

    #[structopt(long = "log-csv", alias = "oc", parse(from_os_str), help = "Log ping results a csv file. [alias: --oc]")]
    pub csv_log_path: Option<PathBuf>,

    #[structopt(long = "log-json", alias = "oj", parse(from_os_str), help = "Log ping results to a json file. [alias: --oj]")]
    pub json_log_path: Option<PathBuf>,

    #[structopt(short = "o", long = "log-text", parse(from_os_str), help = "Log ping results to a text file.")]
    pub text_log_path: Option<PathBuf>,

    #[structopt(short = "r", long, help = "Show ping result scatter map after ping is done.")]
    pub show_result_scatter: bool,

    #[structopt(short = "l", long, help = "Show latency (round trip time) scatter map after ping is done.")]
    pub show_latency_scatter: bool,

    #[structopt(
        short = "b",
        long = "latency-buckets",
        use_delimiter = true,
        help = "If set, bucket ping latency (round trip time) after ping is done. Set to 0.0 to use the default one: [0.1,0.5,1.0,10.0,50.0,100.0,300.0,500.0]"
    )]
    pub latency_buckets: Option<Vec<f64>>,
}

#[derive(Debug, StructOpt, PartialEq)]
pub struct RnpCliQuicPingOptions {
    #[structopt(long, help = "Specify the server name in the QUIC pings. Example: localhost.")]
    pub server_name: Option<String>,

    #[structopt(
        long,
        help = "Enable key logger in TLS for helping packet capture.\nPlease note that it might cause RTT to be slightly larger than the real one, because logging key will also take time."
    )]
    pub log_tls_key: bool,

    #[structopt(
        long = "alpn",
        default_value = "h3-29",
        help = "ALPN protocol used in QUIC. Specify \"none\" to disable ALPN.\nIt is usually h3-<ver> for http/3 or hq-<ver> for specific version of QUIC.\nFor latest IDs, please check here: https://www.iana.org/assignments/tls-extensiontype-values/tls-extensiontype-values.xhtml#alpn-protocol-ids"
    )]
    pub alpn_protocol: String,

    #[structopt(
        long,
        help = "Calculate the RTT by checking the time of before and after doing QUIC connect instead of estimated RTT from QUIC. Not recommended, as this might cause the RTT time to be larger than the real one."
    )]
    pub use_timer_rtt: bool,
}

impl RnpCliOptions {
    pub fn prepare_to_use(&mut self) {
        self.ping_common_options.prepare_to_use(&self.common_options.target);

        if let Some(latency_buckets) = &mut self.output_options.latency_buckets {
            tracing::debug!("Latency bucket set to 0. Use default one.");
            if latency_buckets.len() == 0 || (latency_buckets.len() == 1 && latency_buckets[0] == 0.0) {
                *latency_buckets = vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 50.0, 100.0, 300.0, 500.0];
            }
        }
    }

    pub fn to_ping_runner_config(&self) -> RnpPingRunnerConfig {
        let mut config = RnpPingRunnerConfig {
            worker_config: PingWorkerConfig {
                protocol: self.common_options.protocol.clone(),
                target: self.common_options.target,
                source_ip: self.ping_common_options.source_ip,
                ping_interval: Duration::from_millis(self.ping_common_options.ping_interval_in_ms.into()),
                ping_client_config: PingClientConfig {
                    wait_timeout: Duration::from_millis(self.ping_common_options.wait_timeout_in_ms.into()),
                    time_to_live: self.ping_common_options.time_to_live,
                    check_disconnect: self.ping_common_options.check_disconnect,
                    wait_before_disconnect: Duration::from_millis(self.ping_common_options.wait_before_disconnect_in_ms),
                    server_name: self.quic_options.server_name.as_ref().and_then(|s| Some(s.to_string())),
                    log_tls_key: self.quic_options.log_tls_key,
                    alpn_protocol: if self.quic_options.alpn_protocol.to_uppercase() != String::from("NONE") {
                        Some(self.quic_options.alpn_protocol.clone())
                    } else {
                        None
                    },
                    use_timer_rtt: self.quic_options.use_timer_rtt,
                },
            },
            worker_scheduler_config: PingWorkerSchedulerConfig {
                source_ports: self.ping_common_options.source_ports.as_ref().unwrap().clone(),
                ping_count: None,
                warmup_count: self.ping_common_options.warmup_count,
                parallel_ping_count: self.ping_common_options.parallel_ping_count,
            },
            result_processor_config: PingResultProcessorConfig {
                common_config: PingResultProcessorCommonConfig { quiet_level: self.output_options.quiet_level },
                exit_on_fail: self.ping_common_options.exit_on_fail,
                exit_failure_reason: if self.ping_common_options.exit_on_fail { Some(Arc::new(Mutex::new(None))) } else { None },
                csv_log_path: self.output_options.csv_log_path.clone(),
                json_log_path: self.output_options.json_log_path.clone(),
                text_log_path: self.output_options.text_log_path.clone(),
                show_result_scatter: self.output_options.show_result_scatter,
                show_latency_scatter: self.output_options.show_latency_scatter,
                latency_buckets: self.output_options.latency_buckets.as_ref().and_then(|buckets| Some(buckets.clone())),
            },
            external_ping_client_factory: None,
            extra_ping_result_processors: vec![],
        };

        if !self.ping_common_options.ping_until_stopped {
            config.worker_scheduler_config.ping_count = Some(self.ping_common_options.ping_count);
        }

        return config;
    }
}

impl RnpCliPingCommonOptions {
    pub fn prepare_to_use(&mut self, target: &SocketAddr) {
        if target.is_ipv4() != self.source_ip.is_ipv4() {
            match &self.source_ip {
                IpAddr::V4(source_ip_v4) if *source_ip_v4 == Ipv4Addr::UNSPECIFIED => self.source_ip = IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                IpAddr::V6(source_ip_v6) if *source_ip_v6 == Ipv6Addr::UNSPECIFIED => self.source_ip = IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                _ => panic!("Source IP and Target IP are not both IPv4 or IPv6!"),
            }
        }

        if self.source_ports.is_none() {
            let range_start = rand::thread_rng().gen_range(10000..30000);
            let range_end = range_start + 2000;
            self.source_ports = Some(PortRangeList { ranges: vec![(range_start..=range_end)] });
        }

        if !self.ping_until_stopped && self.ping_count < 1 {
            tracing::warn!("Ping count cannot be less than 1, setting to 1 as minimum.");
            self.ping_count = 1;
        }

        let available_source_port_count = self.source_ports.as_ref().unwrap().calculate_total_port_count();
        if self.parallel_ping_count > available_source_port_count as u32 {
            tracing::warn!(
                "Parallel ping count ({}) is larger than available source port count ({}), to avoid port conflict reducing parallel ping count down to the same as available source port count.",
                self.parallel_ping_count,
                available_source_port_count);

            self.parallel_ping_count = available_source_port_count as u32;
        }

        if self.parallel_ping_count < 1 {
            tracing::warn!("Parallel ping count cannot be 0. Setting to 1 as minimum.");
            self.parallel_ping_count = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rnp::{
        PingClientConfig, PingResultProcessorCommonConfig, PingResultProcessorConfig, PingWorkerConfig, PingWorkerSchedulerConfig,
        RnpPingRunnerConfig, RnpSupportedProtocol, RNP_QUIET_LEVEL_NONE, RNP_QUIET_LEVEL_NO_OUTPUT, RNP_QUIET_LEVEL_NO_PING_RESULT,
    };
    use std::path::PathBuf;
    use std::time::Duration;
    use structopt::StructOpt;

    #[test]
    fn parsing_default_options_should_work() {
        assert_eq!(
            RnpCliOptions {
                common_options: RnpCliCommonOptions { target: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::TCP },
                ping_common_options: RnpCliPingCommonOptions {
                    source_ip: "0.0.0.0".parse().unwrap(),
                    source_ports: None,
                    ping_count: 4,
                    ping_until_stopped: false,
                    warmup_count: 0,
                    wait_timeout_in_ms: 2000,
                    ping_interval_in_ms: 1000,
                    time_to_live: None,
                    check_disconnect: false,
                    wait_before_disconnect_in_ms: 0,
                    parallel_ping_count: 1,
                    exit_on_fail: false,
                },
                quic_options: RnpCliQuicPingOptions {
                    server_name: None,
                    log_tls_key: false,
                    alpn_protocol: String::from("h3-29"),
                    use_timer_rtt: false,
                },
                output_options: RnpCliOutputOptions {
                    quiet_level: RNP_QUIET_LEVEL_NONE,
                    csv_log_path: None,
                    json_log_path: None,
                    text_log_path: None,
                    show_result_scatter: false,
                    show_latency_scatter: false,
                    latency_buckets: None,
                },
            },
            RnpCliOptions::from_iter(&["tp.exe", "10.0.0.1:443"])
        );
    }

    #[test]
    fn parsing_short_options_should_work() {
        assert_eq!(
            RnpCliOptions {
                common_options: RnpCliCommonOptions { target: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::TCP },
                ping_common_options: RnpCliPingCommonOptions {
                    source_ip: "10.0.0.2".parse().unwrap(),
                    source_ports: Some(PortRangeList { ranges: vec![(1024..=2048), (3096..=3096), (3097..=3097)] }),
                    ping_count: 10,
                    ping_until_stopped: true,
                    warmup_count: 0,
                    wait_timeout_in_ms: 1000,
                    ping_interval_in_ms: 1500,
                    time_to_live: None,
                    check_disconnect: true,
                    wait_before_disconnect_in_ms: 0,
                    parallel_ping_count: 10,
                    exit_on_fail: false,
                },
                quic_options: RnpCliQuicPingOptions {
                    server_name: None,
                    log_tls_key: false,
                    alpn_protocol: String::from("h3-29"),
                    use_timer_rtt: false,
                },
                output_options: RnpCliOutputOptions {
                    quiet_level: RNP_QUIET_LEVEL_NO_PING_RESULT,
                    csv_log_path: Some(PathBuf::from("log.csv")),
                    json_log_path: Some(PathBuf::from("log.json")),
                    text_log_path: Some(PathBuf::from("log.txt")),
                    show_result_scatter: true,
                    show_latency_scatter: true,
                    latency_buckets: Some(vec![0.1, 0.5, 1.0, 10.0]),
                },
            },
            RnpCliOptions::from_iter(&[
                "rnp.exe",
                "10.0.0.1:443",
                "-m",
                "tcp",
                "-s",
                "10.0.0.2",
                "--sp",
                "1024-2048,3096,3097",
                "-n",
                "10",
                "-t",
                "-w",
                "1000",
                "-i",
                "1500",
                "-d",
                "-p",
                "10",
                "-q",
                "--oc",
                "log.csv",
                "--oj",
                "log.json",
                "-o",
                "log.txt",
                "-r",
                "-l",
                "-b",
                "0.1,0.5,1.0,10.0",
            ])
        );
    }

    #[test]
    fn parsing_long_options_should_work() {
        assert_eq!(
            RnpCliOptions {
                common_options: RnpCliCommonOptions { target: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::QUIC },
                ping_common_options: RnpCliPingCommonOptions {
                    source_ip: "10.0.0.2".parse().unwrap(),
                    source_ports: Some(PortRangeList { ranges: vec![(1024..=2048), (3096..=3096), (3097..=3097)] }),
                    ping_count: 10,
                    ping_until_stopped: false,
                    warmup_count: 3,
                    wait_timeout_in_ms: 1000,
                    ping_interval_in_ms: 1500,
                    time_to_live: Some(128),
                    check_disconnect: true,
                    wait_before_disconnect_in_ms: 3000,
                    parallel_ping_count: 10,
                    exit_on_fail: true,
                },
                quic_options: RnpCliQuicPingOptions {
                    server_name: Some(String::from("localhost")),
                    log_tls_key: true,
                    alpn_protocol: String::from("hq-29"),
                    use_timer_rtt: true,
                },
                output_options: RnpCliOutputOptions {
                    quiet_level: RNP_QUIET_LEVEL_NO_OUTPUT,
                    csv_log_path: Some(PathBuf::from("log.csv")),
                    json_log_path: Some(PathBuf::from("log.json")),
                    text_log_path: Some(PathBuf::from("log.txt")),
                    show_result_scatter: true,
                    show_latency_scatter: true,
                    latency_buckets: Some(vec![0.1, 0.5, 1.0, 10.0]),
                },
            },
            RnpCliOptions::from_iter(&[
                "rnp.exe",
                "10.0.0.1:443",
                "--mode",
                "quic",
                "--src-ip",
                "10.0.0.2",
                "--src-ports",
                "1024-2048,3096,3097",
                "--count",
                "10",
                "--warmup",
                "3",
                "--timeout",
                "1000",
                "--interval",
                "1500",
                "--ttl",
                "128",
                "--check-disconnect",
                "--wait-before-disconnect",
                "3000",
                "--parallel",
                "10",
                "--exit-on-fail",
                "--server-name",
                "localhost",
                "--log-tls-key",
                "--alpn",
                "hq-29",
                "--use-timer-rtt",
                "-qqq",
                "--log-csv",
                "log.csv",
                "--log-json",
                "log.json",
                "--log-text",
                "log.txt",
                "--show-result-scatter",
                "--show-latency-scatter",
                "--latency-buckets",
                "0.1,0.5,1.0,10.0",
            ])
        );
    }

    #[test]
    fn new_from_cli_options_should_work() {
        assert_eq!(
            RnpPingRunnerConfig {
                worker_config: PingWorkerConfig {
                    protocol: RnpSupportedProtocol::TCP,
                    target: "10.0.0.1:443".parse().unwrap(),
                    source_ip: "10.0.0.2".parse().unwrap(),
                    ping_interval: Duration::from_millis(1500),
                    ping_client_config: PingClientConfig {
                        wait_timeout: Duration::from_millis(1000),
                        time_to_live: Some(128),
                        check_disconnect: false,
                        wait_before_disconnect: Duration::from_millis(2000),
                        server_name: None,
                        log_tls_key: false,
                        alpn_protocol: None,
                        use_timer_rtt: false,
                    },
                },
                worker_scheduler_config: PingWorkerSchedulerConfig {
                    source_ports: PortRangeList { ranges: vec![(1024..=2048), (3096..=3096), (3097..=3097)] },
                    ping_count: Some(4),
                    warmup_count: 1,
                    parallel_ping_count: 1,
                },
                result_processor_config: PingResultProcessorConfig {
                    common_config: PingResultProcessorCommonConfig { quiet_level: RNP_QUIET_LEVEL_NONE },
                    exit_on_fail: false,
                    exit_failure_reason: None,
                    csv_log_path: None,
                    json_log_path: None,
                    text_log_path: None,
                    show_result_scatter: false,
                    show_latency_scatter: false,
                    latency_buckets: None,
                },
                external_ping_client_factory: None,
                extra_ping_result_processors: vec![],
            },
            RnpCliOptions {
                common_options: RnpCliCommonOptions { target: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::TCP },
                ping_common_options: RnpCliPingCommonOptions {
                    ping_count: 4,
                    ping_until_stopped: false,
                    warmup_count: 1,
                    source_ip: "10.0.0.2".parse().unwrap(),
                    source_ports: Some(PortRangeList { ranges: vec![(1024..=2048), (3096..=3096), (3097..=3097)] }),
                    wait_timeout_in_ms: 1000,
                    ping_interval_in_ms: 1500,
                    time_to_live: Some(128),
                    check_disconnect: false,
                    wait_before_disconnect_in_ms: 2000,
                    parallel_ping_count: 1,
                    exit_on_fail: false,
                },
                quic_options: RnpCliQuicPingOptions {
                    server_name: None,
                    log_tls_key: false,
                    alpn_protocol: String::from("none"),
                    use_timer_rtt: false,
                },
                output_options: RnpCliOutputOptions {
                    quiet_level: RNP_QUIET_LEVEL_NONE,
                    csv_log_path: None,
                    json_log_path: None,
                    text_log_path: None,
                    show_result_scatter: false,
                    show_latency_scatter: false,
                    latency_buckets: None,
                },
            }
            .to_ping_runner_config()
        );

        assert_eq!(
            RnpPingRunnerConfig {
                worker_config: PingWorkerConfig {
                    protocol: RnpSupportedProtocol::QUIC,
                    target: "10.0.0.1:443".parse().unwrap(),
                    source_ip: "10.0.0.2".parse().unwrap(),
                    ping_interval: Duration::from_millis(1500),
                    ping_client_config: PingClientConfig {
                        wait_timeout: Duration::from_millis(2000),
                        time_to_live: Some(128),
                        check_disconnect: true,
                        wait_before_disconnect: Duration::from_millis(3000),
                        server_name: Some(String::from("localhost")),
                        log_tls_key: true,
                        alpn_protocol: Some(String::from("h3")),
                        use_timer_rtt: true,
                    },
                },
                worker_scheduler_config: PingWorkerSchedulerConfig {
                    source_ports: PortRangeList { ranges: vec![(1024..=2048), (3096..=3096), (3097..=3097)] },
                    ping_count: None,
                    warmup_count: 3,
                    parallel_ping_count: 1,
                },
                result_processor_config: PingResultProcessorConfig {
                    common_config: PingResultProcessorCommonConfig { quiet_level: RNP_QUIET_LEVEL_NO_PING_RESULT },
                    exit_on_fail: true,
                    exit_failure_reason: Some(Arc::new(Mutex::new(None))),
                    csv_log_path: Some(PathBuf::from("log.csv")),
                    json_log_path: Some(PathBuf::from("log.json")),
                    text_log_path: Some(PathBuf::from("log.txt")),
                    show_result_scatter: true,
                    show_latency_scatter: true,
                    latency_buckets: Some(vec![0.1, 0.5, 1.0, 10.0]),
                },
                external_ping_client_factory: None,
                extra_ping_result_processors: vec![],
            },
            RnpCliOptions {
                common_options: RnpCliCommonOptions { target: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::QUIC },
                ping_common_options: RnpCliPingCommonOptions {
                    ping_count: 4,
                    ping_until_stopped: true,
                    warmup_count: 3,
                    source_ip: "10.0.0.2".parse().unwrap(),
                    source_ports: Some(PortRangeList { ranges: vec![(1024..=2048), (3096..=3096), (3097..=3097)] }),
                    wait_timeout_in_ms: 2000,
                    ping_interval_in_ms: 1500,
                    time_to_live: Some(128),
                    check_disconnect: true,
                    wait_before_disconnect_in_ms: 3000,
                    parallel_ping_count: 1,
                    exit_on_fail: true,
                },
                quic_options: RnpCliQuicPingOptions {
                    server_name: Some(String::from("localhost")),
                    log_tls_key: true,
                    alpn_protocol: String::from("h3"),
                    use_timer_rtt: true,
                },
                output_options: RnpCliOutputOptions {
                    quiet_level: RNP_QUIET_LEVEL_NO_PING_RESULT,
                    csv_log_path: Some(PathBuf::from("log.csv")),
                    json_log_path: Some(PathBuf::from("log.json")),
                    text_log_path: Some(PathBuf::from("log.txt")),
                    show_result_scatter: true,
                    show_latency_scatter: true,
                    latency_buckets: Some(vec![0.1, 0.5, 1.0, 10.0]),
                },
            }
            .to_ping_runner_config()
        );
    }

    #[test]
    fn empty_source_port_in_options_should_be_fixed() {
        let mut opts = RnpCliOptions::from_iter(&["rnp.exe", "10.0.0.1:443"]);
        opts.prepare_to_use();

        assert!(opts.ping_common_options.source_ports.is_some());
        assert_eq!(1, opts.ping_common_options.source_ports.as_ref().unwrap().ranges.len());
        assert_eq!(
            2000,
            opts.ping_common_options.source_ports.as_ref().unwrap().ranges[0].end()
                - opts.ping_common_options.source_ports.as_ref().unwrap().ranges[0].start()
        );
    }

    #[test]
    fn invalid_options_for_ipv4_should_be_fixed() {
        let mut opts = RnpCliOptions::from_iter(&["rnp.exe", "10.0.0.1:443"]);
        opts.ping_common_options.source_ports = None;
        opts.ping_common_options.ping_count = 0;
        opts.ping_common_options.parallel_ping_count = 0;
        opts.output_options.latency_buckets = Some(vec![0.0]);
        opts.prepare_to_use();

        assert_eq!(1, opts.ping_common_options.ping_count);
        assert_eq!(1, opts.ping_common_options.parallel_ping_count);
        assert_eq!(Some(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 50.0, 100.0, 300.0, 500.0]), opts.output_options.latency_buckets);

        opts.ping_common_options.source_ports = Some(PortRangeList { ranges: vec![(1024..=1047)] });
        opts.ping_common_options.parallel_ping_count = 100;
        opts.prepare_to_use();
        assert_eq!(24, opts.ping_common_options.parallel_ping_count);

        opts.ping_common_options.source_ports = Some(PortRangeList { ranges: vec![(1024..=1024), (1025..=1025), (1026..=1026)] });
        opts.ping_common_options.parallel_ping_count = 100;
        opts.prepare_to_use();
        assert_eq!(3, opts.ping_common_options.parallel_ping_count);
    }

    #[test]
    fn invalid_options_for_ipv6_should_be_fixed() {
        let mut opts = RnpCliOptions::from_iter(&["rnp.exe", "[2607:f8b0:400a:80a::200e]:443"]);
        opts.prepare_to_use();

        // If source ip is not set (unspecified/any), we update the IP accordingly to match our target.
        assert!(opts.ping_common_options.source_ip.is_ipv6());
        assert_eq!(Ipv6Addr::UNSPECIFIED, opts.ping_common_options.source_ip);
    }
}
