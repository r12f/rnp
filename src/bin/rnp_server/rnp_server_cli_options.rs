use rnp::{parse_ping_target, RnpStubServerConfig, RnpSupportedProtocol};
use std::net::SocketAddr;
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
            report_interval: Duration::from_millis(1000),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rnp::{RnpStubServerConfig, RnpSupportedProtocol};
    use structopt::StructOpt;

    #[test]
    fn parsing_default_options_should_work() {
        assert_eq!(
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions { server_address: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::TCP },
            },
            RnpServerCliOptions::from_iter(&["rnp_server.exe", "10.0.0.1:443"])
        );
    }

    #[test]
    fn parsing_short_options_should_work() {
        assert_eq!(
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions { server_address: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::TCP },
            },
            RnpServerCliOptions::from_iter(&["rnp_server.exe", "10.0.0.1:443", "-m", "tcp",])
        );
    }

    #[test]
    fn parsing_long_options_should_work() {
        assert_eq!(
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions { server_address: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::QUIC },
            },
            RnpServerCliOptions::from_iter(&["rnp_server.exe", "10.0.0.1:443", "--mode", "quic",])
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
                report_interval: Duration::from_millis(1000),
            },
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions { server_address: "10.0.0.1:443".parse().unwrap(), protocol: RnpSupportedProtocol::TCP },
            }
            .to_stub_server_config()
        );
    }
}
