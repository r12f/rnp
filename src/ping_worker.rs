use crate::ping_clients::ping_client::PingClientPingResultDetails;
use crate::{ping_client_factory, PingClient, PingPortPicker, PingResult, PingWorkerConfig};
use chrono::{offset::Utc, DateTime};
use futures_intrusive::sync::ManualResetEvent;
use socket2::SockAddr;
use std::{net::SocketAddr, sync::Arc, sync::Mutex, io};
use tokio::{sync::mpsc, task, task::JoinHandle};
use std::time::Duration;

pub struct PingWorker {
    id: u32,
    config: Arc<PingWorkerConfig>,
    stop_event: Arc<ManualResetEvent>,
    port_picker: Arc<Mutex<PingPortPicker>>,
    ping_client: Box<dyn PingClient + Send + Sync>,
    result_sender: mpsc::Sender<PingResult>,
    is_warmup_worker: bool,
}

impl PingWorker {
    #[tracing::instrument(
        name = "Starting worker",
        level = "debug",
        skip(config, port_picker, stop_event, result_sender)
    )]
    pub fn run(
        worker_id: u32,
        config: Arc<PingWorkerConfig>,
        port_picker: Arc<Mutex<PingPortPicker>>,
        stop_event: Arc<ManualResetEvent>,
        result_sender: mpsc::Sender<PingResult>,
        is_warmup_worker: bool,
    ) -> JoinHandle<()> {
        let join_handle = task::spawn(async move {
            let ping_client =
                ping_client_factory::new(config.protocol, &config.ping_client_config);

            let mut worker = PingWorker {
                id: worker_id,
                config,
                stop_event,
                port_picker,
                ping_client,
                result_sender,
                is_warmup_worker,
            };
            worker.run_worker_loop().await;

            tracing::debug!("Ping worker loop exited; worker_id={}", worker.id);
        });

        return join_handle;
    }

    #[tracing::instrument(name = "Running worker loop", level = "debug", skip(self), fields(worker_id = %self.id))]
    async fn run_worker_loop(&mut self) {
        loop {
            let source_port = self
                .port_picker
                .lock()
                .expect("Failed getting port picker lock")
                .next();
            match source_port {
                Some(source_port) => self.run_single_ping(source_port).await,
                None => {
                    tracing::debug!("Ping finished, stopping worker; worker_id={}", self.id);
                    return;
                }
            }

            if !self.wait_for_next_schedule().await {
                break;
            }
        }
    }

    #[tracing::instrument(name = "Running single ping", level = "debug", skip(self), fields(worker_id = %self.id))]
    async fn run_single_ping(&mut self, source_port: u16) {
        let source = SockAddr::from(SocketAddr::new(self.config.source_ip, source_port));
        let target = SockAddr::from(self.config.target);
        let ping_time = Utc::now();

        if let Err(prepare_error) = self.ping_client.prepare_for_ping(&source) {
            self.process_ping_client_preparation_error(&ping_time, source_port, prepare_error).await;
            return;
        }

        match self.ping_client.ping(&target) {
            Ok(result) => {
                self.process_ping_client_result(&ping_time, source_port, result)
                    .await
            }
            Err(result) => {
                self.process_ping_client_result(&ping_time, source_port, result)
                    .await
            }
        }
    }

    #[tracing::instrument(name = "Processing ping client single ping preparation error", level = "debug", skip(self), fields(worker_id = %self.id))]
    async fn process_ping_client_preparation_error(
        &self,
        ping_time: &DateTime<Utc>,
        src_port: u16,
        prepare_error: io::Error,
    ) {
        let source = SocketAddr::new(self.config.source_ip, src_port);

        let result = PingResult::new(
            ping_time,
            self.id,
            self.ping_client.protocol(),
            self.config.target,
            source,
            self.is_warmup_worker,
            Duration::from_millis(0),
            Some(prepare_error),
            true,
        );

        self.result_sender.send(result).await.unwrap();
    }

    #[tracing::instrument(name = "Processing ping client single ping result", level = "debug", skip(self), fields(worker_id = %self.id))]
    async fn process_ping_client_result(
        &self,
        ping_time: &DateTime<Utc>,
        src_port: u16,
        ping_result: PingClientPingResultDetails,
    ) {
        let mut source: Option<SocketAddr> =
            ping_result.actual_local_addr.and_then(|x| x.as_socket());

        if source.is_none() {
            source = Some(SocketAddr::new(self.config.source_ip, src_port));
        }

        let result = PingResult::new(
            ping_time,
            self.id,
            self.ping_client.protocol(),
            self.config.target,
            source.unwrap(),
            self.is_warmup_worker,
            ping_result.round_trip_time,
            ping_result.ping_error,
            false,
        );

        self.result_sender.send(result).await.unwrap();
    }

    #[tracing::instrument(name = "Waiting for next schedule", level = "debug", skip(self), fields(worker_id = %self.id))]
    async fn wait_for_next_schedule(&self) -> bool {
        let ping_interval = self.config.ping_interval;
        let result = tokio::time::timeout(ping_interval, self.stop_event.wait()).await;

        // Wait succedded, which means we are signaled to exit.
        if let Ok(_) = result {
            tracing::debug!(
                "Stop event received, stopping worker; worker_id={}",
                self.id
            );
            return false;
        }

        // If not, continue to run.
        return true;
    }
}
