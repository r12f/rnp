use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::PingResult;
use std::collections::BTreeMap;
use tracing;

const COUNT_PER_ROW: u32 = 20;
const SCATTER_SYMBOL_NOT_TESTED: char = '-';
const SCATTER_SYMBOL_PASSED: char = '1';
const SCATTER_SYMBOL_FAILED: char = '0';

struct ResultHits {
    bitmask: u32,
    results: u32,
}

pub struct PingResultProcessorResultScatterLogger {
    ping_history: BTreeMap<u32, ResultHits>,
}

impl PingResultProcessorResultScatterLogger {
    #[tracing::instrument(name = "Creating ping result result scatter logger", level = "debug")]
    pub fn new() -> PingResultProcessorResultScatterLogger {
        return PingResultProcessorResultScatterLogger {
            ping_history: BTreeMap::new(),
        };
    }

    fn get_ping_history_bit_pos(&self, port: u32) -> (u32, u32) {
        let row: u32 = (port / COUNT_PER_ROW) * COUNT_PER_ROW;
        let bit_index = port % COUNT_PER_ROW;
        return (row, bit_index);
    }

    fn convert_result_hits_to_string(hits: &ResultHits) -> String {
        let mut s: String = String::new();

        for index in 0..COUNT_PER_ROW {
            let test_bit = 1 << index;
            let mut output_symbol: char = SCATTER_SYMBOL_NOT_TESTED;
            if hits.bitmask & test_bit != 0 {
                output_symbol = if hits.results & test_bit != 0 {
                    SCATTER_SYMBOL_PASSED
                } else {
                    SCATTER_SYMBOL_FAILED
                };
            }
            s.push(output_symbol);

            if (index != COUNT_PER_ROW - 1) && ((index + 1) % 5 == 0) {
                s.push(' ');
            }
        }

        return s;
    }
}

impl PingResultProcessor for PingResultProcessorResultScatterLogger {
    fn process(&mut self, ping_result: &PingResult) {
        let (row, bit_index) = self.get_ping_history_bit_pos(ping_result.source().port() as u32);
        let bit_mask_bit = 1 << bit_index;
        let success_bit = if let Some(_) = ping_result.error() {
            0
        } else {
            1 << bit_index
        };

        let failure_hits = self
            .ping_history
            .entry(row)
            .or_insert(ResultHits { bitmask: 0, results: 0 });

        failure_hits.bitmask |= bit_mask_bit;
        failure_hits.results |= success_bit;
    }

    fn done(&mut self) {
        println!("\n=== Ping result scatter map ===\n");

        println!("{:>7} | {}", "Src", "Results");
        println!(
            "{:>7} | (\"{}\" = Ok, \"{}\" = Fail, \"{}\" = Not Tested)",
            "Port", SCATTER_SYMBOL_PASSED, SCATTER_SYMBOL_FAILED, SCATTER_SYMBOL_NOT_TESTED
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
    use crate::ping_result_processors::ping_result_processor_result_scatter_logger::{
        PingResultProcessorResultScatterLogger, ResultHits,
    };

    #[test]
    fn convert_result_info_to_string_should_work() {
        let results = vec![
            ResultHits { bitmask: 0, results: 0 },
            ResultHits {
                bitmask: 0b1,
                results: 0b1,
            },
            ResultHits {
                bitmask: 0b1110,
                results: 0b1100,
            },
        ];

        let formatted_results: Vec<String> = results
            .into_iter()
            .map(|x| PingResultProcessorResultScatterLogger::convert_result_hits_to_string(&x))
            .collect();

        assert_eq!(
            vec![
                "----- ----- ----- -----",
                "1---- ----- ----- -----",
                "-011- ----- ----- -----",
            ],
            formatted_results
        );
    }
}
