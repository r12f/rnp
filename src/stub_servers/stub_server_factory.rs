use crate::stub_servers::stub_server_tcp::StubServerTcp;
use crate::*;
use futures_intrusive::sync::ManualResetEvent;
use std::error::Error;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[tracing::instrument(name = "Start running stub server", level = "debug", skip(stop_event))]
pub fn run(config: &RnpStubServerConfig, stop_event: Arc<ManualResetEvent>, server_started_event: Arc<ManualResetEvent>) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
    match config.protocol {
        RnpSupportedProtocol::TCP => return StubServerTcp::run_new(config.clone(), stop_event, server_started_event),
        _ => panic!("Protocol {} is not supported!", config.protocol),
    }
}
