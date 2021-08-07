mod rnp_core_test_mocks;

use rnp::*;
use rnp_core_test_mocks::*;
use tokio::runtime::Runtime;
use std::sync::Arc;
use futures_intrusive::sync::ManualResetEvent;
use std::time::Duration;

#[test]
fn ping_with_rnp_core_should_work() {
    let config = RnpCoreConfig {
        worker_config: PingWorkerConfig {
            protocol: RnpSupportedProtocol::TCP,
            target: "10.0.0.1:443".parse().unwrap(),
            source_ip: "10.0.0.2".parse().unwrap(),
            ping_interval: Duration::from_millis(1500),
            ping_client_config: PingClientConfig {
                wait_timeout: Duration::from_millis(1000),
                time_to_live: Some(128),
                check_disconnect: false,
                server_name: None,
                log_tls_key: false,
                alpn_protocol: None,
                use_timer_rtt: false,
            },
        },
        worker_scheduler_config: PingWorkerSchedulerConfig {
            source_ports: PortRangeList {
                ranges: vec![(1024..=2048), (3096..=3096), (3097..=3097)]
            },
            ping_count: Some(4),
            warmup_count: 1,
            parallel_ping_count: 1,
        },
        result_processor_config: PingResultProcessorConfig {
            no_console_log: false,
            csv_log_path: None,
            json_log_path: None,
            text_log_path: None,
            show_result_scatter: false,
            show_latency_scatter: false,
            latency_buckets: None,
        },
        external_ping_client_factory: Some(|_, _| Some(Box::new(MockPingClient::new(vec![])))),
        extra_ping_result_processors: vec![],
    };

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let stop_event = Arc::new(ManualResetEvent::new(false));
        let mut rp = RnpCore::new(config, stop_event);
        rp.start_running_normal_pings();
        rp.join().await;
    });
}