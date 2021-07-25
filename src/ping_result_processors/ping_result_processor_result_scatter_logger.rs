use crate::*;
use std::collections::BTreeMap;
use tracing;

const COUNT_PER_ROW: u32 = 20;
const SCATTER_SYMBOL_NOT_TESTED_YET: char = '.';
const SCATTER_SYMBOL_PASSED: char = 'O';
const SCATTER_SYMBOL_FAILED: char = 'X';
const SCATTER_SYMBOL_PREPARE_FAILED: char = '-';
const SCATTER_SYMBOL_HANDSHAKE_FAILED: char = 'H';
const SCATTER_SYMBOL_DISCONNECT_FAILED: char = 'D';

pub struct PingResultProcessorResultScatterLogger {
    ping_history: Vec<BTreeMap<u32, Vec<char>>>,
}

impl PingResultProcessorResultScatterLogger {
    #[tracing::instrument(name = "Creating ping result result scatter logger", level = "debug")]
    pub fn new() -> PingResultProcessorResultScatterLogger {
        return PingResultProcessorResultScatterLogger {
            ping_history: vec![BTreeMap::new()],
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
    fn name(&self) -> &'static str {
        "ResultScatterLogger"
    }

    fn process_ping_result(&mut self, ping_result: &PingResult) {
        // Skip warmup pings in analysis.
        if ping_result.is_warmup() {
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

        // Find the last iteration and update the result.
        loop {
            let last_iteration = self
                .ping_history
                .last_mut()
                .expect("Ping history should always be non-empty.");

            let last_iteration_results = last_iteration
                .entry(row)
                .or_insert(vec![SCATTER_SYMBOL_NOT_TESTED_YET; COUNT_PER_ROW as usize]);

            // If the source port is already tested in the last iteration, it means a new iteration is started,
            // hence create a new iteration and update there.
            if last_iteration_results[index] != SCATTER_SYMBOL_NOT_TESTED_YET {
                self.ping_history.push(BTreeMap::new());
                continue;
            }

            last_iteration_results[index] = result;

            break;
        }
    }

    fn rundown(&mut self) {
        println!("\n=== Ping result scatter map ===");
        println!(
            "(\"{}\" = Ok, \"{}\" = Fail, \"{}\" = Not tested yet, \"{}\" = Preparation failed, \"{}\" = App handshake failed, \"{}\" = Disconnect failed)",
            SCATTER_SYMBOL_PASSED, SCATTER_SYMBOL_FAILED, SCATTER_SYMBOL_NOT_TESTED_YET, SCATTER_SYMBOL_PREPARE_FAILED, SCATTER_SYMBOL_HANDSHAKE_FAILED, SCATTER_SYMBOL_DISCONNECT_FAILED
        );

        println!("\n{:>5} | {:>5} | {}", "Iter", "Src", "Results");
        println!("{:>5} | {:>5} | ", "#", "Port");
        println!("{:->6}|{:->8}-0---4-5---9-0---4-5---9-", "", "+");

        for (iteration_index, iteration) in self.ping_history.iter().enumerate() {
            for (port_bucket, result_hits) in iteration {
                print!("{:>5} | {:>5} | ", iteration_index, port_bucket);

                let result = PingResultProcessorResultScatterLogger::convert_result_hits_to_string(
                    result_hits,
                );
                println!("{}", result);
            }
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
