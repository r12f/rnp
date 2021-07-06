use crate::{
    rnp_utils, PingPortPicker, PingResult, PingResultProcessingWorker, PingWorker, RnpCoreConfig, RNP_ABOUT, RNP_AUTHOR,
    RNP_NAME,
};
use futures_intrusive::sync::ManualResetEvent;
use std::sync::{Arc, Mutex};
use tokio::{sync::mpsc, task::JoinHandle};

pub struct RnpCore {
    config: RnpCoreConfig,

    stop_event: Arc<ManualResetEvent>,
    worker_join_handles: Vec<JoinHandle<()>>,
    ping_result_processor_join_handle: Option<JoinHandle<()>>,
}

impl RnpCore {
    #[tracing::instrument(name = "Start running Rnp core", level = "debug", skip(stop_event))]
    pub fn start_run(config: RnpCoreConfig, stop_event: Arc<ManualResetEvent>) -> RnpCore {
        let mut rnp_core = RnpCore {
            config,
            stop_event,
            worker_join_handles: Vec::new(),
            ping_result_processor_join_handle: None,
        };

        rnp_core.start();

        return rnp_core;
    }

    fn start(&mut self) {
        self.log_header_to_console();

        let ping_result_sender = self.create_ping_result_processing_worker();
        self.create_ping_workers(ping_result_sender);
    }

    fn log_header_to_console(&self) {
        println!("{} - {} - {}\n", RNP_NAME, RNP_AUTHOR, RNP_ABOUT);

        println!(
            "Start testing {} {:?}:",
            rnp_utils::format_protocol(self.config.worker_config.protocol),
            self.config.worker_config.target
        );
    }

    #[tracing::instrument(name = "Creating ping result processing worker", level = "debug", skip(self))]
    fn create_ping_result_processing_worker(&mut self) -> mpsc::Sender<PingResult> {
        let mut ping_result_channel_size = self.config.worker_scheduler_config.parallel_ping_count * 2;
        if ping_result_channel_size < 128 {
            ping_result_channel_size = 128;
        }

        let (ping_result_sender, ping_result_receiver) = mpsc::channel(ping_result_channel_size as usize);
        self.ping_result_processor_join_handle = Some(PingResultProcessingWorker::run(
            Arc::new(self.config.result_processor_config.clone()),
            self.stop_event.clone(),
            ping_result_receiver,
        ));

        return ping_result_sender;
    }

    #[tracing::instrument(name = "Creating all ping workers", level = "debug", skip(self, sender))]
    fn create_ping_workers(&mut self, sender: mpsc::Sender<PingResult>) {
        let mut worker_join_handles = Vec::new();

        let source_port_picker = Arc::new(Mutex::new(PingPortPicker::new(
            self.config.worker_scheduler_config.ping_count,
            self.config.worker_scheduler_config.source_port_min,
            self.config.worker_scheduler_config.source_port_max,
            &self.config.worker_scheduler_config.source_port_list,
        )));

        let worker_count = self.config.worker_scheduler_config.parallel_ping_count;
        let shared_worker_config = Arc::new(self.config.worker_config.clone());
        for worker_id in 0..worker_count {
            let worker_join_handle = PingWorker::run(
                worker_id,
                shared_worker_config.clone(),
                source_port_picker.clone(),
                self.stop_event.clone(),
                sender.clone(),
            );
            worker_join_handles.push(worker_join_handle);
        }

        self.worker_join_handles = worker_join_handles;
    }

    #[tracing::instrument(name = "Waiting for all workers to finish", level = "debug", skip(self))]
    pub async fn join(&mut self) {
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
        self.ping_result_processor_join_handle.take().unwrap().await.unwrap();
    }
}
