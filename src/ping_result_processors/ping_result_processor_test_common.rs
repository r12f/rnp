use crate::rnp_test_common;
use crate::ping_result_processors::ping_result_processor::PingResultProcessor;

pub fn run_ping_result_processor_with_test_samples(
    processor: &mut Box<dyn PingResultProcessor + Send + Sync>,
)
{
    let ping_results = rnp_test_common::generate_ping_result_test_samples();

    processor.initialize();
    for ping_result in &ping_results {
        processor.process_ping_result(&ping_result);
    }
    processor.rundown();
}