use crate::{PingResult, PingResultProcessorCommonConfig};

pub trait PingResultProcessor {
    fn name(&self) -> &'static str;

    fn config(&self) -> &PingResultProcessorCommonConfig;
    fn has_quiet_level(&self, quiet_level: i32) -> bool { self.config().quiet_level >= quiet_level }

    fn initialize(&mut self) {}
    fn process_ping_result(&mut self, ping_result: &PingResult);
    fn rundown(&mut self) {}
}
