use rand::Rng;
use rnp::{
    parse_ping_target, PingClientConfig, PingResultProcessorCommonConfig, PingResultProcessorConfig, PingWorkerConfig, PingWorkerSchedulerConfig,
    PortRangeList, RnpPingRunnerConfig, RnpStubServerConfig, RnpSupportedProtocol,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt, PartialEq)]
#[structopt(name = rnp::RNP_NAME, author = rnp::RNP_AUTHOR, about = rnp::RNP_ABOUT)]
pub struct RnpServerCliOptions {
    #[structopt(flatten)]
    pub common_options: RnpServerCliCommonOptions,
}

#[derive(Debug, StructOpt, PartialEq)]
pub struct RnpServerCliCommonOptions {
    #[structopt(short = "m", long = "mode", default_value = "TCP", help = "Specify protocol to use.")]
    pub protocol: RnpSupportedProtocol,

    #[structopt(parse(try_from_str = parse_ping_target), help = "Server address.")]
    pub server_address: SocketAddr,
}

impl RnpServerCliOptions {
    pub fn prepare_to_use(&mut self) {}

    pub fn to_stub_server_config(self) -> RnpStubServerConfig {
        return RnpStubServerConfig {
            protocol: self.common_options.protocol,
            server_address: self.common_options.server_address,
            close_on_accept: false,
            sleep_before_write: None,
            write_chunk_size: 0,
            write_count_limit: None,
            report_interval: Duration::ZERO,
        };
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
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions { server_address: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::TCP },
            },
            RnpCliOptions::from_iter(&["rnp_server.exe", "10.0.0.1:443"])
        );
    }

    #[test]
    fn parsing_short_options_should_work() {
        assert_eq!(
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions { server_address: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::TCP },
            },
            RnpCliOptions::from_iter(&["rnp_server.exe", "10.0.0.1:443", "-m", "tcp",])
        );
    }

    #[test]
    fn parsing_long_options_should_work() {
        assert_eq!(
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions { server_address: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::QUIC },
            },
            RnpCliOptions::from_iter(&["rnp_server.exe", "10.0.0.1:443", "--mode", "quic",])
        );
    }

    #[test]
    fn new_from_cli_options_should_work() {
        assert_eq!(
            RnpStubServerConfig {
                protocol: RnpSupportedProtocol::TCP,
                server_address: "10.0.0.1:443".parse().unwrap(),
                close_on_accept: false,
                sleep_before_write: None,
                write_chunk_size: 0,
                write_count_limit: None,
                report_interval: Default::default()
            },
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions { server_address: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::TCP },
            }
            .to_ping_runner_config()
        );
    }
}
