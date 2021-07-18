use rand::Rng;
use rnp::{PingClientConfig, PingResultProcessorConfig, PingWorkerConfig, PingWorkerSchedulerConfig, RnpCoreConfig, RnpSupportedProtocol};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt, PartialOrd, PartialEq)]
#[structopt(name = rnp::RNP_NAME, author = rnp::RNP_AUTHOR, about = rnp::RNP_ABOUT)]
pub struct RnpCliOptions {
    pub target: SocketAddr,

    #[structopt(short = "m", long = "mode", default_value = "TCP", help = "Specify protocol to use.")]
    pub protocol: RnpSupportedProtocol,

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

    #[structopt(
        long = "src-port",
        use_delimiter = true,
        help = "Source port list to use in ping."
    )]
    pub source_port_list: Option<Vec<u16>>,

    #[structopt(short = "n", long = "count", default_value = "4", help = "Ping count.")]
    pub ping_count: u32,

    #[structopt(short = "t", help = "Ping until stopped.")]
    pub ping_until_stopped: bool,

    #[structopt(long = "warmup", default_value = "1", help = "Warm up ping count.")]
    pub warmup_count: u32,

    #[structopt(
        short = "w",
        long = "timeout",
        default_value = "2000",
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

    #[structopt(long = "use-fin", help = "Use regular TCP disconnect (w/ FIN) instead of RST.")]
    pub use_fin_in_tcp_ping: bool,

    #[structopt(
        short = "p",
        long = "parallel",
        default_value = "1",
        help = "Count of pings running in parallel."
    )]
    pub parallel_ping_count: u32,

    #[structopt(long, help = "Specify the server name in the pings, such as QUIC.")]
    pub server_name: Option<String>,

    #[structopt(long, help = "Enable key logger in TLS for helping packet capture. Please note that it might cause RTT to be larger than the real one, because logging key will also take time.")]
    pub log_tls_key: bool,

    #[structopt(long = "alpn", help = "ALPN protocol used in QUIC, it is usually h3-<ver> for http/3 or hq-<ver> for specific version of QUIC. For latest IDs, please check here: https://www.iana.org/assignments/tls-extensiontype-values/tls-extensiontype-values.xhtml#alpn-protocol-ids")]
    pub alpn_protocol: Option<String>,

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
        help = "Show latency (round trip time) scatter map after ping is done."
    )]
    pub show_latency_scatter: bool,

    #[structopt(
        short = "b",
        long = "latency-buckets",
        use_delimiter = true,
        help = "If set, bucket ping latency (round trip time) after ping is done. Set to 0.0 to use the default one: [0.1,0.5,1.0,10.0,50.0,100.0,300.0,500.0]"
    )]
    pub latency_buckets: Option<Vec<f64>>,
}

impl RnpCliOptions {
    pub fn prepare_to_use(&mut self) {
        if self.target.is_ipv4() != self.source_ip.is_ipv4() {
            match &self.source_ip {
                IpAddr::V4(source_ip_v4) if *source_ip_v4 == Ipv4Addr::UNSPECIFIED => {
                    self.source_ip = IpAddr::V6(Ipv6Addr::UNSPECIFIED)
                }
                IpAddr::V6(source_ip_v6) if *source_ip_v6 == Ipv6Addr::UNSPECIFIED => {
                    self.source_ip = IpAddr::V4(Ipv4Addr::UNSPECIFIED)
                }
                _ => panic!("Source IP and Target IP are not both IPv4 or IPv6!"),
            }
        }
        if self.source_port_min.is_none() {
            self.source_port_min = Some(rand::thread_rng().gen_range(10000..30000));
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

        let available_source_port_count = match &self.source_port_list {
            Some(port_list) => port_list.len() as u32,
            None => self.source_port_max.unwrap() as u32 - self.source_port_min.unwrap() as u32 + 1,
        };

        if self.parallel_ping_count > available_source_port_count {
            tracing::warn!(
                "Parallel ping count ({}) is larger than available source port count ({}), to avoid port conflict reducing parallel ping count down to the same as available source port count.",
                self.parallel_ping_count,
                available_source_port_count);

            self.parallel_ping_count = available_source_port_count;
        }

        if self.parallel_ping_count < 1 {
            tracing::warn!("Parallel ping count cannot be 0. Setting to 1 as minimum.");
            self.parallel_ping_count = 1;
        }

        if let Some(latency_buckets) = &mut self.latency_buckets {
            tracing::debug!("Latency bucket set to 0. Use default one.");
            if latency_buckets.len() == 0
                || (latency_buckets.len() == 1 && latency_buckets[0] == 0.0)
            {
                *latency_buckets = vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 50.0, 100.0, 300.0, 500.0];
            }
        }
    }

    pub fn to_rnp_core_config(&self) -> RnpCoreConfig {
        let mut config = RnpCoreConfig {
            worker_config: PingWorkerConfig {
                protocol: self.protocol,
                target: self.target,
                source_ip: self.source_ip,
                ping_interval: Duration::from_millis(self.ping_interval_in_ms.into()),
                ping_client_config: PingClientConfig {
                    wait_timeout: Duration::from_millis(self.wait_timeout_in_ms.into()),
                    time_to_live: self.time_to_live,
                    use_fin_in_tcp_ping: self.use_fin_in_tcp_ping,
                    server_name: self.server_name.as_ref().and_then(|s| Some(s.to_string())),
                    log_tls_key: self.log_tls_key,
                    alpn_protocol: self.alpn_protocol.as_ref().and_then(|s| Some(s.to_string())),
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
                warmup_count: self.warmup_count,
                parallel_ping_count: self.parallel_ping_count,
            },
            result_processor_config: PingResultProcessorConfig {
                no_console_log: self.no_console_log,
                csv_log_path: self.csv_log_path.clone(),
                json_log_path: self.json_log_path.clone(),
                text_log_path: self.text_log_path.clone(),
                show_result_scatter: self.show_result_scatter,
                show_latency_scatter: self.show_latency_scatter,
                latency_buckets: self.latency_buckets.as_ref().and_then(|buckets| Some(buckets.clone())),
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
    use rnp::{PingClientConfig, PingResultProcessorConfig, PingWorkerConfig, PingWorkerSchedulerConfig, RnpCoreConfig, RnpSupportedProtocol};
    use std::path::PathBuf;
    use std::time::Duration;
    use structopt::StructOpt;

    #[test]
    fn parsing_default_options_should_work() {
        assert_eq!(
            RnpCliOptions {
                target: "10.0.0.1:443".parse().unwrap(),
                protocol: RnpSupportedProtocol::TCP,
                source_ip: "0.0.0.0".parse().unwrap(),
                source_port_min: None,
                source_port_max: None,
                source_port_list: None,
                ping_count: 4,
                ping_until_stopped: false,
                warmup_count: 1,
                wait_timeout_in_ms: 2000,
                ping_interval_in_ms: 1000,
                time_to_live: None,
                use_fin_in_tcp_ping: false,
                parallel_ping_count: 1,
                server_name: None,
                log_tls_key: false,
                alpn_protocol: None,
                no_console_log: false,
                csv_log_path: None,
                json_log_path: None,
                text_log_path: None,
                show_result_scatter: false,
                show_latency_scatter: false,
                latency_buckets: None,
            },
            RnpCliOptions::from_iter(&["tp.exe", "10.0.0.1:443"])
        );
    }

    #[test]
    fn parsing_short_options_should_work() {
        assert_eq!(
            RnpCliOptions {
                target: "10.0.0.1:443".parse().unwrap(),
                protocol: RnpSupportedProtocol::TCP,
                source_ip: "10.0.0.2".parse().unwrap(),
                source_port_min: None,
                source_port_max: None,
                source_port_list: None,
                ping_count: 10,
                ping_until_stopped: true,
                warmup_count: 1,
                wait_timeout_in_ms: 1000,
                ping_interval_in_ms: 1500,
                time_to_live: None,
                use_fin_in_tcp_ping: false,
                parallel_ping_count: 10,
                server_name: None,
                log_tls_key: false,
                alpn_protocol: None,
                no_console_log: true,
                csv_log_path: None,
                json_log_path: None,
                text_log_path: None,
                show_result_scatter: true,
                show_latency_scatter: true,
                latency_buckets: Some(vec![0.1, 0.5, 1.0, 10.0]),
            },
            RnpCliOptions::from_iter(&[
                "rnp.exe",
                "10.0.0.1:443",
                "-m",
                "tcp",
                "-s",
                "10.0.0.2",
                "-n",
                "10",
                "-t",
                "-w",
                "1000",
                "-i",
                "1500",
                "-p",
                "10",
                "-q",
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
                target: "10.0.0.1:443".parse().unwrap(),
                protocol: RnpSupportedProtocol::QUIC,
                source_ip: "10.0.0.2".parse().unwrap(),
                source_port_min: Some(1024),
                source_port_max: Some(2048),
                source_port_list: Some(vec![1024, 1025, 1026]),
                ping_count: 10,
                ping_until_stopped: false,
                warmup_count: 3,
                wait_timeout_in_ms: 1000,
                ping_interval_in_ms: 1500,
                time_to_live: Some(128),
                use_fin_in_tcp_ping: true,
                parallel_ping_count: 10,
                server_name: Some(String::from("localhost")),
                log_tls_key: true,
                alpn_protocol: Some(String::from("hq-29")),
                no_console_log: true,
                csv_log_path: Some(PathBuf::from("log.csv")),
                json_log_path: Some(PathBuf::from("log.json")),
                text_log_path: Some(PathBuf::from("log.txt")),
                show_result_scatter: true,
                show_latency_scatter: true,
                latency_buckets: Some(vec![0.1, 0.5, 1.0, 10.0]),
            },
            RnpCliOptions::from_iter(&[
                "rnp.exe",
                "10.0.0.1:443",
                "--mode",
                "quic",
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
                "--warmup",
                "3",
                "--timeout",
                "1000",
                "--interval",
                "1500",
                "--ttl",
                "128",
                "--use-fin",
                "--parallel",
                "10",
                "--server-name",
                "localhost",
                "--log-tls-key",
                "--alpn",
                "hq-29",
                "--no-console-log",
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
            RnpCoreConfig {
                worker_config: PingWorkerConfig {
                    protocol: RnpSupportedProtocol::TCP,
                    target: "10.0.0.1:443".parse().unwrap(),
                    source_ip: "10.0.0.2".parse().unwrap(),
                    ping_interval: Duration::from_millis(1500),
                    ping_client_config: PingClientConfig {
                        wait_timeout: Duration::from_millis(1000),
                        time_to_live: Some(128),
                        use_fin_in_tcp_ping: false,
                        server_name: None,
                        log_tls_key: false,
                        alpn_protocol: None
                    },
                },
                worker_scheduler_config: PingWorkerSchedulerConfig {
                    source_port_min: 1024,
                    source_port_max: 2047,
                    source_port_list: Some(vec![1024, 1025, 1026]),
                    ping_count: Some(4),
                    warmup_count: 1,
                    parallel_ping_count: 1,
                },
                result_processor_config: PingResultProcessorConfig {
                    no_console_log: false,
                    csv_log_path: None,
                    json_log_path: None,
                    text_log_path: None,
                    show_result_scatter: false,
                    show_latency_scatter: false,
                    latency_buckets: None,
                },
            },
            RnpCliOptions {
                target: "10.0.0.1:443".parse().unwrap(),
                protocol: RnpSupportedProtocol::TCP,
                ping_count: 4,
                ping_until_stopped: false,
                warmup_count: 1,
                source_ip: "10.0.0.2".parse().unwrap(),
                source_port_min: Some(1024),
                source_port_max: Some(2047),
                source_port_list: Some(vec![1024, 1025, 1026]),
                wait_timeout_in_ms: 1000,
                ping_interval_in_ms: 1500,
                time_to_live: Some(128),
                use_fin_in_tcp_ping: false,
                parallel_ping_count: 1,
                server_name: None,
                log_tls_key: false,
                alpn_protocol: None,
                no_console_log: false,
                csv_log_path: None,
                json_log_path: None,
                text_log_path: None,
                show_result_scatter: false,
                show_latency_scatter: false,
                latency_buckets: None,
            }
            .to_rnp_core_config()
        );

        assert_eq!(
            RnpCoreConfig {
                worker_config: PingWorkerConfig {
                    protocol: RnpSupportedProtocol::QUIC,
                    target: "10.0.0.1:443".parse().unwrap(),
                    source_ip: "10.0.0.2".parse().unwrap(),
                    ping_interval: Duration::from_millis(1500),
                    ping_client_config: PingClientConfig {
                        wait_timeout: Duration::from_millis(2000),
                        time_to_live: Some(128),
                        use_fin_in_tcp_ping: true,
                        server_name: Some(String::from("localhost")),
                        log_tls_key: true,
                        alpn_protocol: Some(String::from("h3")),
                    },
                },
                worker_scheduler_config: PingWorkerSchedulerConfig {
                    source_port_min: 1024,
                    source_port_max: 2047,
                    source_port_list: Some(vec![1024, 1025, 1026]),
                    ping_count: None,
                    warmup_count: 3,
                    parallel_ping_count: 1,
                },
                result_processor_config: PingResultProcessorConfig {
                    no_console_log: true,
                    csv_log_path: Some(PathBuf::from("log.csv")),
                    json_log_path: Some(PathBuf::from("log.json")),
                    text_log_path: Some(PathBuf::from("log.txt")),
                    show_result_scatter: true,
                    show_latency_scatter: true,
                    latency_buckets: Some(vec![0.1, 0.5, 1.0, 10.0]),
                },
            },
            RnpCliOptions {
                target: "10.0.0.1:443".parse().unwrap(),
                protocol: RnpSupportedProtocol::QUIC,
                ping_count: 4,
                ping_until_stopped: true,
                warmup_count: 3,
                source_ip: "10.0.0.2".parse().unwrap(),
                source_port_min: Some(1024),
                source_port_max: Some(2047),
                source_port_list: Some(vec![1024, 1025, 1026]),
                wait_timeout_in_ms: 2000,
                ping_interval_in_ms: 1500,
                time_to_live: Some(128),
                use_fin_in_tcp_ping: true,
                parallel_ping_count: 1,
                server_name: Some(String::from("localhost")),
                log_tls_key: true,
                alpn_protocol: Some(String::from("h3")),
                no_console_log: true,
                csv_log_path: Some(PathBuf::from("log.csv")),
                json_log_path: Some(PathBuf::from("log.json")),
                text_log_path: Some(PathBuf::from("log.txt")),
                show_result_scatter: true,
                show_latency_scatter: true,
                latency_buckets: Some(vec![0.1, 0.5, 1.0, 10.0]),
            }
            .to_rnp_core_config()
        );
    }

    #[test]
    fn empty_source_port_in_options_should_be_fixed() {
        let mut opts = RnpCliOptions::from_iter(&["rnp.exe", "10.0.0.1:443"]);
        opts.prepare_to_use();

        assert!(opts.source_port_min.is_some());
        assert!(opts.source_port_max.is_some());
        assert_eq!(
            10000,
            opts.source_port_max.unwrap() - opts.source_port_min.unwrap()
        );
    }

    #[test]
    fn invalid_options_for_ipv4_should_be_fixed() {
        let mut opts = RnpCliOptions::from_iter(&["rnp.exe", "10.0.0.1:443"]);
        opts.source_port_min = Some(2048);
        opts.source_port_max = Some(1024);
        opts.ping_count = 0;
        opts.parallel_ping_count = 0;
        opts.latency_buckets = Some(vec![0.0]);
        opts.prepare_to_use();

        assert_eq!(1024, opts.source_port_min.unwrap());
        assert_eq!(2048, opts.source_port_max.unwrap());
        assert_eq!(1, opts.ping_count);
        assert_eq!(1, opts.parallel_ping_count);
        assert_eq!(
            Some(vec![
                0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 50.0, 100.0, 300.0, 500.0
            ]),
            opts.latency_buckets
        );

        opts.source_port_min = Some(1024);
        opts.source_port_max = Some(1047);
        opts.parallel_ping_count = 100;
        opts.prepare_to_use();
        assert_eq!(24, opts.parallel_ping_count);

        opts.source_port_min = None;
        opts.source_port_max = None;
        opts.source_port_list = Some(vec![1024, 1025, 1026]);
        opts.parallel_ping_count = 100;
        opts.prepare_to_use();
        assert_eq!(3, opts.parallel_ping_count);
    }

    #[test]
    fn invalid_options_for_ipv6_should_be_fixed() {
        let mut opts = RnpCliOptions::from_iter(&["rnp.exe", "[2607:f8b0:400a:80a::200e]:443"]);
        opts.prepare_to_use();

        // If source ip is not set (unspecified/any), we update the IP accordingly to match our target.
        assert!(opts.source_ip.is_ipv6());
        assert_eq!(Ipv6Addr::UNSPECIFIED, opts.source_ip);
    }
}
