use crate::*;
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
    /// Take the config and stop event and generate the RnpCore which runs all the pings.
    ///
    /// # Arguments
    ///
    /// * `config`: The configuration of Rnp.
    /// * `stop_event`: The event to signal Rnp to stop.
    ///
    /// returns: RnpCore
    ///
    /// # Examples
    ///
    /// ```
    /// use rnp::*;
    /// use std::time::Duration;
    /// use std::sync::Arc;
    /// use futures_intrusive::sync::ManualResetEvent;
    /// use tokio::runtime::Runtime;
    ///
    /// let config = RnpCoreConfig {
    ///     worker_config: PingWorkerConfig {
    ///         protocol: RnpSupportedProtocol::TCP,
    ///         target: "10.0.0.1:443".parse().unwrap(),
    ///         source_ip: "10.0.0.2".parse().unwrap(),
    ///         ping_interval: Duration::from_millis(1500),
    ///         ping_client_config: PingClientConfig {
    ///             wait_timeout: Duration::from_millis(1000),
    ///             time_to_live: Some(128),
    ///             check_disconnect: false,
    ///             server_name: None,
    ///             log_tls_key: false,
    ///             alpn_protocol: None,
    ///             use_timer_rtt: false,
    ///         },
    ///     },
    ///     worker_scheduler_config: PingWorkerSchedulerConfig {
    ///         source_port_min: 1024,
    ///         source_port_max: 2047,
    ///         source_port_list: Some(vec![1024, 1025, 1026]),
    ///         ping_count: Some(4),
    ///         warmup_count: 1,
    ///         parallel_ping_count: 1,
    ///     },
    ///     result_processor_config: PingResultProcessorConfig {
    ///         no_console_log: false,
    ///         csv_log_path: None,
    ///         json_log_path: None,
    ///         text_log_path: None,
    ///         show_result_scatter: false,
    ///         show_latency_scatter: false,
    ///         latency_buckets: None,
    ///     },
    ///     external_ping_client_factory: None,
    ///     extra_ping_result_processors: vec![],
    /// };
    ///
    /// let rt = Runtime::new().unwrap();
    /// rt.block_on(async {
    ///     let stop_event = Arc::new(ManualResetEvent::new(false));
    ///     let core = RnpCore::new(config, stop_event);
    /// });
    ///
    /// ```
    #[tracing::instrument(name = "Start running Rnp core", level = "debug", skip(stop_event))]
    pub fn new(mut config: RnpCoreConfig, stop_event: Arc<ManualResetEvent>) -> RnpCore {
        // Move all extra ping result processors into another Vec for initializing result processing worker.
        // Otherwise RnpCoreConfig will be partially moved and results in compile error.
        let mut extra_ping_result_processors = Vec::new();
        extra_ping_result_processors.append(&mut config.extra_ping_result_processors);

        let (result_sender, ping_result_processor_join_handle) =
            RnpCore::create_ping_result_processing_worker(
                config.result_processor_config.clone(),
                extra_ping_result_processors,
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

    #[tracing::instrument(name = "Creating ping result processing worker", level = "debug", skip(extra_ping_result_processors))]
    fn create_ping_result_processing_worker(
        result_processor_config: PingResultProcessorConfig,
        extra_ping_result_processors: Vec<Box<dyn PingResultProcessor + Send + Sync>>,
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
            extra_ping_result_processors,
            stop_event,
            ping_result_receiver,
        );

        return (ping_result_sender, ping_result_processor_join_handle);
    }

    fn log_header_to_console(&self) {
        println!(
            "Start testing {} {:?}:",
            self.config.worker_config.protocol, self.config.worker_config.target
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

    #[tracing::instrument(name = "Start running normal pings", level = "debug", skip(self))]
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
