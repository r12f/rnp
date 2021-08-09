use crate::{ping_result_processors::ping_result_processor_factory, PingResult, PingResultProcessor, PingResultProcessorConfig};
use futures_intrusive::sync::ManualResetEvent;
use std::sync::{Arc, Mutex};
use tokio::{sync::mpsc, task, task::JoinHandle};

pub struct PingResultProcessingWorker {
    stop_event: Arc<ManualResetEvent>,

    receiver: mpsc::Receiver<PingResult>,
    processors: Vec<Box<dyn PingResultProcessor + Send + Sync>>,

    ping_stop_event: Arc<ManualResetEvent>,
    exit_on_fail: bool,
    exit_failure_reason: Option<Arc<Mutex<Option<PingResult>>>>,
}

impl PingResultProcessingWorker {
    #[requires(config.exit_on_fail -> config.exit_failure_reason.is_some())]
    pub fn run(
        config: Arc<PingResultProcessorConfig>,
        extra_ping_result_processors: Vec<Box<dyn PingResultProcessor + Send + Sync>>,
        stop_event: Arc<ManualResetEvent>,
        ping_stop_event: Arc<ManualResetEvent>,
        receiver: mpsc::Receiver<PingResult>,
    ) -> JoinHandle<()> {
        let join_handle = task::spawn(async move {
            let processors = ping_result_processor_factory::new(&config, extra_ping_result_processors);
            let mut worker = PingResultProcessingWorker {
                stop_event,
                receiver,
                processors,
                ping_stop_event,
                exit_on_fail: config.exit_on_fail,
                exit_failure_reason: config.exit_failure_reason.clone(),
            };
            worker.run_worker().await;
        });

        return join_handle;
    }

    #[tracing::instrument(name = "Running ping result processing worker loop", level = "debug", skip(self), fields(processor_count = %self.processors.len()))]
    async fn run_worker(&mut self) {
        self.initialize_all_processors();
        self.run_result_processing_loop().await;
        self.signal_all_processors_done();
    }

    #[tracing::instrument(name = "Preparing all ping result processors", level = "debug", skip(self), fields(processor_count = %self.processors.len()))]
    fn initialize_all_processors(&mut self) {
        for processor in &mut self.processors {
            processor.initialize();
        }
    }

    #[tracing::instrument(name = "Running ping result processing loop.", level = "debug", skip(self), fields(processor_count = %self.processors.len()))]
    async fn run_result_processing_loop(&mut self) {
        loop {
            tokio::select! {
                Some(ping_result) = self.receiver.recv() => {
                    self.process_ping_result(&ping_result);
                }

                _ = self.stop_event.wait() => {
                    tracing::debug!("Stop event received, stopping ping result processing worker");
                    break;
                }

                else => {
                    tracing::debug!("Channel closed, stopping ping result processing worker");
                    break;
                }
            }
        }
    }

    #[tracing::instrument(name = "Processing ping result", level = "debug", skip(self), fields(processor_count = %self.processors.len()))]
    fn process_ping_result(&mut self, ping_result: &PingResult) {
        for processor in &mut self.processors {
            processor.process_ping_result(ping_result);
        }

        if self.exit_on_fail && !ping_result.is_succeeded() {
            self.exit_failure_reason.unwrap();
        }
    }

    #[tracing::instrument(name = "Signal all ping result processors done", level = "debug", skip(self), fields(processor_count = %self.processors.len()))]
    fn signal_all_processors_done(&mut self) {
        for processor in &mut self.processors {
            processor.rundown();
        }
    }
}
