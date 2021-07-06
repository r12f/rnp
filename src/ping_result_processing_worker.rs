use crate::{PingResult, PingResultProcessorConfig, PingResultProcessor, ping_result_processors::ping_result_processor_factory};
use futures_intrusive::sync::ManualResetEvent;
use std::sync::Arc;
use tokio::{sync::mpsc, task, task::JoinHandle};

pub struct PingResultProcessingWorker {
    stop_event: Arc<ManualResetEvent>,
    receiver: mpsc::Receiver<PingResult>,
    processors: Vec<Box<dyn PingResultProcessor + Send + Sync>>,
}

impl PingResultProcessingWorker {
    pub fn run(
        config: Arc<PingResultProcessorConfig>,
        stop_event: Arc<ManualResetEvent>,
        receiver: mpsc::Receiver<PingResult>,
    ) -> JoinHandle<()> {
        let join_handle = task::spawn(async move {
            let processors = ping_result_processor_factory::new(&config);
            let mut worker = PingResultProcessingWorker {
                stop_event,
                receiver,
                processors,
            };

            worker.run_worker_loop().await;
        });

        return join_handle;
    }

    #[tracing::instrument(name = "Running ping result processing worker loop", level = "debug", skip(self), fields(processor_count = %self.processors.len()))]
    async fn run_worker_loop(&mut self) {
        self.prepare_all_processors();

        while let Some(ping_result) = self.receiver.recv().await {
            self.process_ping_result(&ping_result);

            if self.stop_event.is_set() {
                tracing::debug!("Stop event received, stopping ping result processing worker");
                break;
            }
        }

        self.signal_all_processors_done();
    }

    #[tracing::instrument(name = "Preparing all ping result processors", level = "debug", skip(self), fields(processor_count = %self.processors.len()))]
    fn prepare_all_processors(&mut self) {
        for processor in &mut self.processors {
            processor.prepare();
        }
    }

    #[tracing::instrument(name = "Processing ping result", level = "debug", skip(self), fields(processor_count = %self.processors.len()))]
    fn process_ping_result(&mut self, ping_result: &PingResult) {
        for processor in &mut self.processors {
            processor.process(ping_result);
        }
    }

    #[tracing::instrument(name = "Signal all ping result processors done", level = "debug", skip(self), fields(processor_count = %self.processors.len()))]
    fn signal_all_processors_done(&mut self) {
        for processor in &mut self.processors {
            processor.done();
        }
    }
}
