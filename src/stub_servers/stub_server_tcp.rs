use crate::StubServerConfig;
use std::error::Error;
use std::sync::Arc;
use futures_intrusive::sync::ManualResetEvent;
use tokio::task::JoinHandle;
use std::net::TcpListener;

pub struct StubServerTcp {
    config: StubServerConfig,
}

impl StubServerTcp {
    pub fn run_new(config: StubServerConfig, stop_event: Arc<ManualResetEvent>) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
        return tokio::spawn(async move {
            let mut server = StubServerTcp { config };

            tokio::select! {
                res = server.run() => { return res; }
                _ = stop_event.wait() => { return Ok(()); }
            }
        });
    }

    async fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(&self.config.server_address)?;
        for stream in listener.incoming() {
        }
        return Ok(());
    }
}