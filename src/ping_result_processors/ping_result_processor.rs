use crate::{PingResult, PingResultProcessorCommonConfig};

pub trait PingResultProcessor {
    fn name(&self) -> &'static str;
    fn config(&self) -> &PingResultProcessorCommonConfig;

    fn initialize(&mut self) {}
    fn process_ping_result(&mut self, ping_result: &PingResult);
    fn rundown(&mut self) {}
}

#[macro_export]
macro_rules! prp_log {
    ($self:ident, $($arg:tt)*) => {
        if ($self.config().quiet_level == RNP_QUIET_LEVEL_NONE) {
            print!($($arg)*)
        }
    };
}
