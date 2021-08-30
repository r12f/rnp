use rnp::{parse_ping_target, RnpStubServerConfig, RnpSupportedProtocol};
use std::net::SocketAddr;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt, PartialEq)]
#[structopt(name = rnp::RNP_SERVER_NAME, author = rnp::RNP_AUTHOR, about = rnp::RNP_ABOUT)]
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

    #[structopt(short = "r", long = "report-interval", default_value = "1000", help = "The interval between each status report in milliseconds.")]
    pub report_interval_in_ms: u64,

    #[structopt(long, alias = "coa", help = "Close connection as soon as accepting it. [alias: --coa]")]
    pub close_on_accept: bool,

    #[structopt(short = "w", long, default_value = "0", help = "If not 0, after connection is established, write to connection.")]
    pub write_chunk_size: usize,

    #[structopt(long, alias = "wc", default_value = "1", help = "How many writes to perform, after connection is established. [alias: --wc]")]
    pub write_count_limit: u32,

    #[structopt(
        long = "write-delay",
        alias = "wd",
        default_value = "0",
        help = "When write back is enabled, sleep in milliseconds before write back. [alias: --wd]"
    )]
    pub sleep_before_write_in_ms: u64,
}

impl RnpServerCliOptions {
    pub fn prepare_to_use(&mut self) {}

    pub fn to_stub_server_config(self) -> RnpStubServerConfig {
        return RnpStubServerConfig {
            protocol: self.common_options.protocol,
            server_address: self.common_options.server_address,
            close_on_accept: self.common_options.close_on_accept,
            sleep_before_write: Duration::from_millis(self.common_options.sleep_before_write_in_ms),
            write_chunk_size: self.common_options.write_chunk_size,
            write_count_limit: self.common_options.write_count_limit,
            report_interval: Duration::from_millis(self.common_options.report_interval_in_ms),
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
                common_options: RnpServerCliCommonOptions {
                    protocol: RnpSupportedProtocol::TCP,
                    server_address: "10.0.0.1:443".parse().unwrap(),
                    report_interval_in_ms: 1000,
                    close_on_accept: false,
                    write_chunk_size: 0,
                    write_count_limit: 1,
                    sleep_before_write_in_ms: 0
                },
            },
            RnpServerCliOptions::from_iter(&["rnp_server.exe", "10.0.0.1:443"])
        );
    }

    #[test]
    fn parsing_short_options_should_work() {
        assert_eq!(
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions {
                    protocol: RnpSupportedProtocol::TCP,
                    server_address: "10.0.0.1:443".parse().unwrap(),
                    report_interval_in_ms: 2000,
                    close_on_accept: true,
                    write_chunk_size: 1024,
                    write_count_limit: 10,
                    sleep_before_write_in_ms: 1000,
                },
            },
            RnpServerCliOptions::from_iter(&[
                "rnp_server.exe",
                "10.0.0.1:443",
                "-m",
                "tcp",
                "-r",
                "2000",
                "--coa",
                "-w",
                "1024",
                "--wc",
                "10",
                "--wd",
                "1000"
            ])
        );
    }

    #[test]
    fn parsing_long_options_should_work() {
        assert_eq!(
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions {
                    protocol: RnpSupportedProtocol::QUIC,
                    server_address: "10.0.0.1:443".parse().unwrap(),
                    report_interval_in_ms: 3000,
                    close_on_accept: true,
                    write_chunk_size: 2048,
                    write_count_limit: 20,
                    sleep_before_write_in_ms: 2000,
                },
            },
            RnpServerCliOptions::from_iter(&[
                "rnp_server.exe",
                "10.0.0.1:443",
                "--mode",
                "quic",
                "--report-interval",
                "3000",
                "--close-on-accept",
                "--write-chunk-size",
                "2048",
                "--write-count-limit",
                "20",
                "--write-delay",
                "2000"
            ])
        );
    }

    #[test]
    fn new_from_cli_options_should_work() {
        assert_eq!(
            RnpStubServerConfig {
                protocol: RnpSupportedProtocol::TCP,
                server_address: "10.0.0.1:443".parse().unwrap(),
                report_interval: Duration::from_millis(1000),
                close_on_accept: true,
                write_chunk_size: 2000,
                write_count_limit: 3000,
                sleep_before_write: Duration::from_millis(4000),
            },
            RnpServerCliOptions {
                common_options: RnpServerCliCommonOptions {
                    protocol: RnpSupportedProtocol::TCP,
                    server_address: "10.0.0.1:443".parse().unwrap(),
                    report_interval_in_ms: 1000,
                    close_on_accept: true,
                    write_chunk_size: 2000,
                    write_count_limit: 3000,
                    sleep_before_write_in_ms: 4000,
                },
            }
            .to_stub_server_config()
        );
    }
}
