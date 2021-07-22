use crate::ping_result_processors::ping_result_processor::PingResultProcessor;
use crate::PingResult;
use std::time::Duration;
use tracing;
use contracts::requires;

pub struct PingResultProcessorLatencyBucketLogger {
    buckets_in_us: Vec<u128>,

    total_hit_count: u32,
    bucket_hit_counts: Vec<u32>,
    timed_out_hit_count: u32,
    failed_hit_count: u32,
}

impl PingResultProcessorLatencyBucketLogger {
    #[tracing::instrument(name = "Creating ping result latency bucket logger", level = "debug")]
    #[allow(unreachable_code)]
    #[requires(buckets.len() >= 1)]
    pub fn new(buckets: &Vec<f64>) -> PingResultProcessorLatencyBucketLogger {
        // The buckets from settings are treated as separators, so the real buckets are:
        // - 0->The first bucket defined in settings
        // - whatever defined in settings
        // - the last bucket to max (without timed out)
        // So in our normalized bucket, we treat the separator as upper bound.
        //
        // Then we use other 2 more buckets to track the following 2 scenarios specifically:
        // - Timed out
        // - Failed
        let mut normalized_buckets = vec![];
        buckets
            .into_iter()
            .for_each(|x| normalized_buckets.push((x * 1000.0) as u128));
        normalized_buckets.push(u128::MAX);

        let normalized_bucket_count = normalized_buckets.len();
        return PingResultProcessorLatencyBucketLogger {
            buckets_in_us: normalized_buckets,
            total_hit_count: 0,
            bucket_hit_counts: vec![0; normalized_bucket_count],
            timed_out_hit_count: 0,
            failed_hit_count: 0,
        };
    }

    fn update_statistics(&mut self, ping_result: &PingResult) {
        // Skip warmup pings in analysis.
        if ping_result.is_warmup() {
            return;
        }

        // Skip preparation errors in analysis, since it is not a remote issue.
        if ping_result.is_preparation_error() {
            return;
        }

        self.total_hit_count += 1;

        if ping_result.is_timed_out() {
            self.timed_out_hit_count += 1;
        } else if let Some(_) = ping_result.error() {
            self.failed_hit_count += 1;
        } else {
            self.track_latency_in_buckets(&ping_result.round_trip_time());
        }
    }

    fn track_latency_in_buckets(&mut self, latency: &Duration) {
        // find the bucket from min to max
        for (bucket_index, bucket_time_upper_bound_in_us) in self.buckets_in_us.iter().enumerate() {
            if latency.as_micros() < *bucket_time_upper_bound_in_us {
                self.bucket_hit_counts[bucket_index] += 1;
                return;
            }
        }

        unreachable!();
    }
}

impl PingResultProcessor for PingResultProcessorLatencyBucketLogger {
    fn process_ping_result(&mut self, ping_result: &PingResult) {
        self.update_statistics(ping_result);
    }

    fn rundown(&mut self) {
        println!("\n=== Latency buckets (in milliseconds) ===\n");
        println!("{:>15} | {}", "Latency Range", "Count");
        println!("{:->17}------------ ", "+");

        for (bucket_index, bucket_time_upper_bound_in_us) in self.buckets_in_us.iter().enumerate() {
            let bucket_range = if bucket_index <  self.buckets_in_us.len() - 1 {
                format!("< {:.2}ms", *bucket_time_upper_bound_in_us as f64 / 1000.0)
            } else {
                format!(">= {:.2}ms", self.buckets_in_us[bucket_index - 1] as f64 / 1000.0)
            };

            println!(
                "{:>15} | {}",
                bucket_range, self.bucket_hit_counts[bucket_index]
            );
        }

        println!("{:>15} | {}", "Timed Out", self.timed_out_hit_count);
        println!("{:>15} | {}", "Failed", self.failed_hit_count);
        println!("{:->17}------------ ", "+");
        println!("{:>15} | {}", "Total", self.total_hit_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rnp_test_common;

    #[test]
    fn latency_bucket_logger_should_work() {
        let ping_results = rnp_test_common::generate_ping_result_test_samples();

        let mut logger =
            PingResultProcessorLatencyBucketLogger::new(&vec![0.1, 0.5, 1.0, 10.0, 50.0, 100.0]);
        ping_results
            .iter()
            .for_each(|x| logger.update_statistics(x));

        assert_eq!(3, logger.total_hit_count);
        assert_eq!(1, logger.timed_out_hit_count);
        assert_eq!(1, logger.failed_hit_count);
    }
}
