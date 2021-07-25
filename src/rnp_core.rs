use crate::{
    PingPortPicker, PingResult, PingResultProcessingWorker, PingResultProcessorConfig,
    PingWorker, RnpCoreConfig,
};
use futures_intrusive::sync::ManualResetEvent;
use std::sync::{Arc, Mutex};
use tokio::{sync::mpsc, task::JoinHandle};

pub struct RnpCore {
    config: RnpCoreConfig,

    stop_event: Arc<ManualResetEvent>,
    worker_join_handles: Vec<JoinHandle<()>>,
    ping_result_processor_join_handle: Option<JoinHandle<()>>,
    result_sender: mpsc::Sender<PingResult>,
}

impl RnpCore {
    #[tracing::instrument(name = "Start running Rnp core", level = "debug", skip(stop_event))]
    pub fn new(config: RnpCoreConfig, stop_event: Arc<ManualResetEvent>) -> RnpCore {
        let (result_sender, ping_result_processor_join_handle) =
            RnpCore::create_ping_result_processing_worker(
                config.result_processor_config.clone(),
                config.worker_scheduler_config.parallel_ping_count,
                stop_event.clone(),
            );

        let rnp_core = RnpCore {
            config,
            stop_event,
            worker_join_handles: Vec::new(),
            ping_result_processor_join_handle: Some(ping_result_processor_join_handle),
            result_sender,
        };

        rnp_core.log_header_to_console();

        return rnp_core;
    }

    #[tracing::instrument(name = "Creating ping result processing worker", level = "debug")]
    fn create_ping_result_processing_worker(
        result_processor_config: PingResultProcessorConfig,
        parallel_ping_count: u32,
        stop_event: Arc<ManualResetEvent>,
    ) -> (mpsc::Sender<PingResult>, JoinHandle<()>) {
        let mut ping_result_channel_size = parallel_ping_count * 2;
        if ping_result_channel_size < 128 {
            ping_result_channel_size = 128;
        }

        let (ping_result_sender, ping_result_receiver) =
            mpsc::channel(ping_result_channel_size as usize);
        let ping_result_processor_join_handle = PingResultProcessingWorker::run(
            Arc::new(result_processor_config),
            stop_event,
            ping_result_receiver,
        );

        return (ping_result_sender, ping_result_processor_join_handle);
    }

    fn log_header_to_console(&self) {
        println!(
            "Start testing {} {:?}:",
            self.config.worker_config.protocol,
            self.config.worker_config.target
        );
    }

    #[tracing::instrument(name = "Running warmup pings", level = "debug", skip(self))]
    pub async fn run_warmup_pings(&mut self) {
        let warmup_count = self.config.worker_scheduler_config.warmup_count;
        if warmup_count == 0 {
            tracing::debug!("Warmup count is 0, skip warmup.");
            return;
        }

        tracing::debug!("Creating warmup worker.");
        let source_port_picker = Arc::new(Mutex::new(PingPortPicker::new(
            Some(self.config.worker_scheduler_config.warmup_count),
            self.config.worker_scheduler_config.source_port_min,
            self.config.worker_scheduler_config.source_port_max,
            &self.config.worker_scheduler_config.source_port_list,
            0,
        )));

        let mut worker_join_handles = self.create_ping_workers_with_options(
            1, // Warmup always use only 1 worker.
            source_port_picker,
            true,
        );

        tracing::debug!("Waiting for warmup worker to stop.");
        for join_handle in &mut worker_join_handles {
            join_handle.await.unwrap();
        }

        tracing::debug!("Warmup ping completed!");
    }

    #[tracing::instrument(
        name = "Start running normal pings",
        level = "debug",
        skip(self)
    )]
    pub fn start_running_normal_pings(&mut self) {
        if self.stop_event.is_set() {
            tracing::debug!("Stop event is signaled, skip running normal pings.");
            return;
        }

        // When doing normal pings, we need to skip the ports we have used for warmups, because we
        // need to give them time for OS to recycle the ports. If we use them again immediately,
        // we might see TCP connect retry causing 1 extra second delay on the TTL.
        let warmup_count = self.config.worker_scheduler_config.warmup_count;
        let adjusted_ping_count = match self.config.worker_scheduler_config.ping_count {
            None => None, // None means pings forever (infinite), hence infinite + warmup count = infinite.
            Some(ping_count) => Some(ping_count + warmup_count),
        };

        let source_port_picker = Arc::new(Mutex::new(PingPortPicker::new(
            adjusted_ping_count,
            self.config.worker_scheduler_config.source_port_min,
            self.config.worker_scheduler_config.source_port_max,
            &self.config.worker_scheduler_config.source_port_list,
            warmup_count,
        )));

        let worker_count = self.config.worker_scheduler_config.parallel_ping_count;
        self.worker_join_handles =
            self.create_ping_workers_with_options(worker_count, source_port_picker, false);
    }

    fn create_ping_workers_with_options(
        &mut self,
        worker_count: u32,
        source_port_picker: Arc<Mutex<PingPortPicker>>,
        is_warmup_worker: bool,
    ) -> Vec<JoinHandle<()>> {
        let mut worker_join_handles = Vec::new();

        let shared_worker_config = Arc::new(self.config.worker_config.clone());
        for worker_id in 0..worker_count {
            let worker_join_handle = PingWorker::run(
                worker_id,
                shared_worker_config.clone(),
                self.config.external_ping_client_factory.clone(),
                source_port_picker.clone(),
                self.stop_event.clone(),
                self.result_sender.clone(),
                is_warmup_worker,
            );
            worker_join_handles.push(worker_join_handle);
        }

        return worker_join_handles;
    }

    #[tracing::instrument(
        name = "Waiting for RNP core to be stopped.",
        level = "debug",
        skip(self)
    )]
    pub async fn join(&mut self) {
        tracing::debug!("Waiting for all workers to be stopped.");
        for join_handle in &mut self.worker_join_handles {
            join_handle.await.unwrap();
        }
        self.worker_join_handles.clear();
        tracing::debug!("All workers are stopped.");

        if !self.stop_event.is_set() {
            tracing::debug!(
                "All ping jobs are completed and all workers are stopped. Signal result processor to exit."
            );
            self.stop_event.set();
        }

        tracing::debug!("Waiting for result processor to be stopped.");
        self.ping_result_processor_join_handle
            .take()
            .unwrap()
            .await
            .unwrap();
        tracing::debug!("Result processor stopped.");
    }
}
