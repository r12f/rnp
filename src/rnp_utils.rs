use socket2::Protocol;
use std::fs::{self, File};
use std::path::PathBuf;

pub fn format_protocol(protocol: Protocol) -> &'static str {
    match protocol {
        Protocol::TCP => "TCP",
        Protocol::UDP => "UDP",
        Protocol::ICMPV4 => "ICMP",
        Protocol::ICMPV6 => "ICMPv6",
        _ => "Unknown,",
    }
}

pub fn create_log_file(log_path_buf: &PathBuf) -> File {
    let log_path = log_path_buf.as_path();
    match log_path.parent() {
        Some(log_folder) => fs::create_dir_all(log_folder)
            .expect(&format!("Failed to create log folder: {}", log_folder.display())),
        None => (), // current folder.
    }

    let log_file = match File::create(&log_path) {
        Err(e) => panic!("Failed to create log file: {}: {}", log_path.display(), e),
        Ok(file) => file,
    };

    return log_file;
}
