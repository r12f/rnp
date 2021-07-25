use crate::PingResult;

pub trait PingResultProcessor {
    fn name(&self) -> &'static str;
    fn initialize(&mut self) {}
    fn process_ping_result(&mut self, ping_result: &PingResult);
    fn rundown(&mut self) {}
}
