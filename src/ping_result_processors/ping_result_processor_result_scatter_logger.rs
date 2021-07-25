use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::PingResult;
use std::collections::BTreeMap;
use tracing;
use crate::ping_clients::ping_client::{PingClientError, PingClientWarning};

const COUNT_PER_ROW: u32 = 20;
const SCATTER_SYMBOL_NOT_TESTED_YET: char = '.';
const SCATTER_SYMBOL_PASSED: char = 'O';
const SCATTER_SYMBOL_FAILED: char = 'X';
const SCATTER_SYMBOL_PREPARE_FAILED: char = '-';
const SCATTER_SYMBOL_HANDSHAKE_FAILED: char = 'H';
const SCATTER_SYMBOL_DISCONNECT_FAILED: char = 'D';

pub struct PingResultProcessorResultScatterLogger {
    ping_history: BTreeMap<u32, Vec<char>>,
}

impl PingResultProcessorResultScatterLogger {
    #[tracing::instrument(name = "Creating ping result result scatter logger", level = "debug")]
    pub fn new() -> PingResultProcessorResultScatterLogger {
        return PingResultProcessorResultScatterLogger {
            ping_history: BTreeMap::new(),
        };
    }

    fn get_ping_history_position(&self, port: u32) -> (u32, usize) {
        let row: u32 = (port / COUNT_PER_ROW) * COUNT_PER_ROW;
        let index = port % COUNT_PER_ROW;
        return (row, index as usize);
    }

    fn convert_result_hits_to_string(hits: &Vec<char>) -> String {
        let mut s: String = String::new();

        for index in 0..COUNT_PER_ROW {
            s.push(hits[index as usize]);

            if (index != COUNT_PER_ROW - 1) && ((index + 1) % 5 == 0) {
                s.push(' ');
            }
        }

        return s;
    }
}

impl PingResultProcessor for PingResultProcessorResultScatterLogger {
    fn name(&self) -> &'static str { "ResultScatterLogger" }

    fn process_ping_result(&mut self, ping_result: &PingResult) {
        // Skip warmup pings in analysis.
        if ping_result.is_warmup() {
            return;
        }

        // Skip preparation errors in analysis, since it is not a remote issue.
        if ping_result.is_preparation_error() {
            return;
        }

        let (row, index) = self.get_ping_history_position(ping_result.source().port() as u32);
        let result = if let Some(e) = ping_result.error() {
            match e {
                PingClientError::PreparationFailed(_) => SCATTER_SYMBOL_PREPARE_FAILED,
                PingClientError::PingFailed(_) => SCATTER_SYMBOL_FAILED,
            }
        } else if let Some(e) = ping_result.warning() {
            match e {
                PingClientWarning::AppHandshakeFailed(_) => SCATTER_SYMBOL_HANDSHAKE_FAILED,
                PingClientWarning::DisconnectFailed(_) => SCATTER_SYMBOL_DISCONNECT_FAILED,
            }
        } else {
            SCATTER_SYMBOL_PASSED
        };

        let results = self
            .ping_history
            .entry(row)
            .or_insert(vec![SCATTER_SYMBOL_NOT_TESTED_YET; 20]);

        results[index] = result;
    }

    fn rundown(&mut self) {
        println!("\n=== Ping result scatter map ===\n");

        println!("{:>7} | {}", "Src", "Results");
        println!(
            "{:>7} | (\"{}\" = Ok, \"{}\" = Fail, \"{}\" = Not tested yet, \"{}\" = Preparation failed, \"{}\" = App handshake failed, \"{}\" = Disconnect failed)",
            "Port", SCATTER_SYMBOL_PASSED, SCATTER_SYMBOL_FAILED, SCATTER_SYMBOL_NOT_TESTED_YET, SCATTER_SYMBOL_PREPARE_FAILED, SCATTER_SYMBOL_HANDSHAKE_FAILED, SCATTER_SYMBOL_DISCONNECT_FAILED
        );
        println!("{:->9}-0---4-5---9-0---4-5---9-------------------", "+");

        for (port_bucket, result_hits) in &self.ping_history {
            print!("{:>7} | ", port_bucket);

            let result = PingResultProcessorResultScatterLogger::convert_result_hits_to_string(result_hits);
            println!("{}", result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_result_info_to_string_should_work() {
        let mut results = vec![
            vec![SCATTER_SYMBOL_NOT_TESTED_YET; 20],
            vec![SCATTER_SYMBOL_NOT_TESTED_YET; 20],
            vec![SCATTER_SYMBOL_NOT_TESTED_YET; 20],
        ];

        results[1][0] = SCATTER_SYMBOL_PASSED;
        results[2][1] = SCATTER_SYMBOL_FAILED;
        results[2][2] = SCATTER_SYMBOL_PREPARE_FAILED;
        results[2][3] = SCATTER_SYMBOL_HANDSHAKE_FAILED;
        results[2][4] = SCATTER_SYMBOL_DISCONNECT_FAILED;

        let formatted_results: Vec<String> = results
            .into_iter()
            .map(|x| PingResultProcessorResultScatterLogger::convert_result_hits_to_string(&x))
            .collect();

        assert_eq!(
            vec![
                "..... ..... ..... .....",
                "O.... ..... ..... .....",
                ".X-HD ..... ..... .....",
            ],
            formatted_results
        );
    }
}
