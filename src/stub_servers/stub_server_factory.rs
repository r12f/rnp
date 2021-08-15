use crate::stub_servers::stub_server_tcp::StubServerTcp;
use crate::*;
use futures_intrusive::sync::ManualResetEvent;
use std::error::Error;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub fn run(config: &RnpStubServerConfig, stop_event: Arc<ManualResetEvent>) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
    match config.protocol {
        RnpSupportedProtocol::TCP => return StubServerTcp::run_new(config.clone(), stop_event),
        _ => panic!("Protocol {} is not supported!", config.protocol),
    }
}
