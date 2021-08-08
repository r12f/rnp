mod rnp_core_test_mocks;

use futures_intrusive::sync::ManualResetEvent;
use rnp::*;
use rnp_core_test_mocks::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use pretty_assertions::assert_eq;

#[test]
fn ping_with_rnp_core_should_work() {
    let actual_ping_results = Arc::new(Mutex::new(Vec::<MockPingClientResult>::new()));
    let config = create_mock_rnp_config(actual_ping_results.clone(), 6, 3, 1);
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let stop_event = Arc::new(ManualResetEvent::new(false));
        let mut rp = RnpCore::new(config, stop_event);
        rp.run_warmup_pings().await;
        rp.start_running_normal_pings();
        rp.join().await;
    });

    let results = actual_ping_results.lock().unwrap();
    assert_eq!(vec![
        MockPingClientResult::Success(Duration::from_millis(10)),
        MockPingClientResult::Timeout,
        MockPingClientResult::PreparationFailed,
        MockPingClientResult::Success(Duration::from_millis(10)),
        MockPingClientResult::Timeout,
        MockPingClientResult::PreparationFailed,
        MockPingClientResult::PingFailed,
        MockPingClientResult::AppHandshakeFailed(Duration::from_millis(20)),
        MockPingClientResult::DisconnectFailed(Duration::from_millis(30)),
    ], *results);
}

#[test]
fn ping_with_rnp_core_stress_should_work() {
    let actual_ping_results = Arc::new(Mutex::new(Vec::<MockPingClientResult>::new()));
    let config = create_mock_rnp_config(actual_ping_results.clone(), 1000, 0, 10);
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let stop_event = Arc::new(ManualResetEvent::new(false));
        let mut rp = RnpCore::new(config, stop_event);
        rp.run_warmup_pings().await;
        rp.start_running_normal_pings();
        rp.join().await;
    });

    let results = actual_ping_results.lock().unwrap();
    assert_eq!(1000, results.len());
}

#[test]
fn ping_with_rnp_core_stop_event_should_work() {
    let actual_ping_results = Arc::new(Mutex::new(Vec::<MockPingClientResult>::new()));
    let config = create_mock_rnp_config(actual_ping_results.clone(), 1000, 0, 10);
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let stop_event = Arc::new(ManualResetEvent::new(false));
        let stop_event_clone = stop_event.clone();

        let mut rp = RnpCore::new(config, stop_event);
        rp.run_warmup_pings().await;

        rp.start_running_normal_pings();
        tokio::spawn(async move { stop_event_clone.set() });

        rp.join().await;
    });
}

fn create_mock_rnp_config(actual_ping_results: Arc<Mutex<Vec<MockPingClientResult>>>, ping_count: u32, warmup_count: u32, parallel_ping_count: u32) -> RnpCoreConfig {
    RnpCoreConfig {
        worker_config: PingWorkerConfig {
            protocol: RnpSupportedProtocol::TCP,
            target: "10.0.0.1:443".parse().unwrap(),
            source_ip: "10.0.0.2".parse().unwrap(),
            ping_interval: Duration::from_millis(0),
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
                ranges: vec![(1024..=2048)],
            },
            ping_count: Some(ping_count),
            warmup_count,
            parallel_ping_count,
        },
        result_processor_config: PingResultProcessorConfig {
            common_config: PingResultProcessorCommonConfig {
                quiet_level: RNP_QUIET_LEVEL_NONE,
            },
            csv_log_path: None,
            json_log_path: None,
            text_log_path: None,
            show_result_scatter: false,
            show_latency_scatter: false,
            latency_buckets: None,
        },
        external_ping_client_factory: Some(|_, config| {
            Some(Box::new(MockPingClient::new(config, vec![
                MockPingClientResult::Success(Duration::from_millis(10)),
                MockPingClientResult::Timeout,
                MockPingClientResult::PreparationFailed,
                MockPingClientResult::PingFailed,
                MockPingClientResult::AppHandshakeFailed(Duration::from_millis(20)),
                MockPingClientResult::DisconnectFailed(Duration::from_millis(30)),
            ])))
        }),
        extra_ping_result_processors: vec![
            Box::new(MockPingResultProcessor::new(actual_ping_results)),
        ],
    }
}