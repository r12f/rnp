use crate::PingResult;

pub trait PingResultProcessor {
    fn prepare(&mut self) {}
    fn process(&mut self, ping_result: &PingResult);
    fn done(&mut self) {}
}