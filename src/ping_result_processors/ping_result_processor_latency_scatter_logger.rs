use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::PingResult;
use std::collections::BTreeMap;
use tracing;

const COUNT_PER_ROW: usize = 10;
const SCATTER_SYMBOL_NOT_TESTED: &str = "   -   ";
const SCATTER_SYMBOL_FAILED: &str = "   X   ";

struct LatencyHits {
    bitmask: u32,
    results: Vec<f64>,
}

pub struct PingResultProcessorLatencyScatterLogger {
    ping_history: BTreeMap<usize, LatencyHits>,
}

impl PingResultProcessorLatencyScatterLogger {
    #[tracing::instrument(name = "Creating ping result latency scatter logger", level = "debug")]
    pub fn new() -> PingResultProcessorLatencyScatterLogger {
        return PingResultProcessorLatencyScatterLogger {
            ping_history: BTreeMap::new(),
        };
    }

    fn get_ping_history_item_pos(&self, port: u32) -> (usize, usize) {
        let row = (port as usize / COUNT_PER_ROW) * COUNT_PER_ROW;
        let col = port as usize % COUNT_PER_ROW;
        return (row, col);
    }

    fn convert_latency_hits_to_string(hits: &LatencyHits) -> String {
        let mut s: String = String::new();

        for (index, latency) in hits.results.iter().enumerate() {
            let test_bit = 1 << index;
            let formatted_latency: String;

            let mut output_symbol: &str = SCATTER_SYMBOL_NOT_TESTED;
            if hits.bitmask & test_bit != 0 {
                if latency.is_nan() {
                    output_symbol = SCATTER_SYMBOL_FAILED;
                } else {
                    formatted_latency = format!("{:^7.2}", latency);
                    output_symbol = &formatted_latency;
                }
            }

            s.push_str(output_symbol);
        }

        return s;
    }
}

impl PingResultProcessor for PingResultProcessorLatencyScatterLogger {
    fn name(&self) -> &'static str {
        "LatencyScatterLogger"
    }

    fn process_ping_result(&mut self, ping_result: &PingResult) {
        // Skip warmup pings in analysis.
        if ping_result.is_warmup() {
            return;
        }

        // Skip preparation errors in analysis, since it is not a remote issue.
        if ping_result.is_preparation_error() {
            return;
        }

        let (row, col) = self.get_ping_history_item_pos(ping_result.source().port() as u32);
        let bit_mask_bit = 1 << col;

        let failure_hits = self.ping_history.entry(row).or_insert(LatencyHits {
            bitmask: 0,
            results: vec![f64::NAN; COUNT_PER_ROW],
        });

        failure_hits.bitmask |= bit_mask_bit;

        if let None = ping_result.error() {
            failure_hits.results[col] = ping_result.round_trip_time().as_micros() as f64 / 1000.0;
        }
    }

    fn rundown(&mut self) {
        println!("\n=== Latency scatter map (in milliseconds) ===\n");

        println!(
            "{:>10} | {} (\"{}\" = Fail, \"{}\" = Not Tested)",
            "Src Port",
            "Results",
            SCATTER_SYMBOL_FAILED.trim(),
            SCATTER_SYMBOL_NOT_TESTED.trim()
        );
        println!(
            "{:->12}-{:-^7.2}{:-^7.2}{:-^7.2}{:-^7.2}{:-^7.2}{:-^7.2}{:-^7.2}{:-^7.2}{:-^7.2}{:-^7.2}",
            "+", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        );

        for (port_bucket, latency_hits) in &self.ping_history {
            print!("{:>10} | ", port_bucket);

            let result = PingResultProcessorLatencyScatterLogger::convert_latency_hits_to_string(
                latency_hits,
            );
            println!("{}", result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_result_info_to_string_should_work() {
        let results = vec![
            LatencyHits {
                bitmask: 0,
                results: vec![f64::NAN; COUNT_PER_ROW],
            },
            LatencyHits {
                bitmask: 0b1,
                results: vec![12.34, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            },
            LatencyHits {
                bitmask: 0b11110,
                results: vec![
                    0.0,
                    f64::NAN,
                    12.34,
                    345.67,
                    234.56,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                ],
            },
        ];

        let formatted_results: Vec<String> = results
            .into_iter()
            .map(|x| PingResultProcessorLatencyScatterLogger::convert_latency_hits_to_string(&x))
            .collect();

        assert_eq!(
            vec![
                "   -      -      -      -      -      -      -      -      -      -   ",
                " 12.34    -      -      -      -      -      -      -      -      -   ",
                "   -      X    12.34 345.67 234.56    -      -      -      -      -   ",
            ],
            formatted_results
        );
    }
}
