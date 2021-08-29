use std::fs::{self, File};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

pub fn create_log_file(log_path_buf: &PathBuf) -> File {
    let log_path = log_path_buf.as_path();
    match log_path.parent() {
        Some(log_folder) => fs::create_dir_all(log_folder).expect(&format!("Failed to create log folder: {}", log_folder.display())),
        None => (), // current folder.
    }

    let log_file = match File::create(&log_path) {
        Err(e) => panic!("Failed to create log file: {}: {}", log_path.display(), e),
        Ok(file) => file,
    };

    return log_file;
}

pub fn parse_ping_target(input: &str) -> Result<SocketAddr, String> {
    let ip: IpAddr;
    let mut port: u16 = 80;

    let last_bracket_index = input.rfind("]");
    let last_colon_index = input.rfind(":");

    let mut ip_str: &str;
    let mut port_str: Option<&str> = None;

    // IPv6
    if let Some(last_bracket_index) = last_bracket_index {
        if let Some(last_colon_index) = last_colon_index {
            if last_colon_index < last_bracket_index {
                // If last colon is before last bracket, then port is not specified. E.g. [::1]
                ip_str = input;
            } else {
                // If port colon is specified, but port is not. E.g. [::1]:. In this case, we default to 80, otherwise we parse it.
                if last_colon_index + 1 < input.len() {
                    port_str = Some(&input[(last_colon_index + 1)..]);
                }

                ip_str = &input[..last_colon_index];
            }
        } else {
            // No port is specified.
            ip_str = input;
        }

        if ip_str.len() < 2 || &ip_str[0..1] != "[" || &ip_str[ip_str.len() - 1..] != "]" {
            return Err(format!("Invalid IP \"{}\" found in ping target \"{}\"", ip_str, input));
        }

        ip_str = &ip_str[1..ip_str.len() - 1];
    }
    // IPv4
    else {
        if let Some(last_colon_index) = last_colon_index {
            // If port colon is specified, but port is not. E.g. 127.0.0.1:. In this case, we default to 80, otherwise we parse it.
            if last_colon_index + 1 < input.len() {
                port_str = Some(&input[(last_colon_index + 1)..]);
            }

            ip_str = &input[..last_colon_index];
        } else {
            // No port is specified.
            ip_str = input;
        }

        // If domain is specified, it will be recognized as a IPv4 IP.
        for c in ip_str.chars() {
            if !c.is_numeric() && c != '.' {
                return Err(format!(
                    "Invalid IP \"{}\" found in ping target \"{}\"\n\n\
                        NOTICE: \"{}\" looks like a domain name and pinging a domain name is explicitly banned. \
                        This is because DNS could return different IP address for the same domain name, \
                        which misleads people when collaborating on network issues. If it is a domain, \
                        please run the following command and and choose a IP to ping, otherwise please \
                        fix the ip and try again:\
                        \n\n    nslookup {}\n",
                    ip_str, input, ip_str, ip_str
                ));
            }
        }
    }

    ip = IpAddr::from_str(ip_str).map_err(|_| format!("Invalid IP \"{}\" found in ping target \"{}\"", ip_str, input))?;

    if let Some(port_str) = port_str {
        port = u16::from_str(port_str).map_err(|_| format!("Invalid port \"{}\" found in ping target \"{}\"", port_str, input))?;
    }

    return Ok(SocketAddr::new(ip, port));
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parsing_ping_target_should_work() {
        assert_eq!(Ok("10.0.0.1:80".parse().unwrap()), parse_ping_target("10.0.0.1"));
        assert_eq!(Ok("10.0.0.1:80".parse().unwrap()), parse_ping_target("10.0.0.1:"));
        assert_eq!(Ok("10.0.0.1:443".parse().unwrap()), parse_ping_target("10.0.0.1:443"));
        assert_eq!(Ok("[::1]:80".parse().unwrap()), parse_ping_target("[::1]"));
        assert_eq!(Ok("[::1]:80".parse().unwrap()), parse_ping_target("[::1]:"));
        assert_eq!(Ok("[::1]:443".parse().unwrap()), parse_ping_target("[::1]:443"));

        assert!(parse_ping_target(":").is_err());
        assert!(parse_ping_target(":443").is_err());
        assert!(parse_ping_target("[").is_err());
        assert!(parse_ping_target("[:").is_err());
        assert!(parse_ping_target("[:443").is_err());
        assert!(parse_ping_target("]").is_err());
        assert!(parse_ping_target("]:").is_err());
        assert!(parse_ping_target("]:443").is_err());
        assert!(parse_ping_target("[]").is_err());

        assert!(parse_ping_target("www.google.com").is_err());
        assert!(parse_ping_target("www.google.com:443").is_err());
    }
}
