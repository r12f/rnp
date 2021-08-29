use futures_intrusive::sync::ManualResetEvent;
use rnp::{PingRunnerCore, RNP_ABOUT, RNP_AUTHOR, RNP_NAME, RNP_QUIET_LEVEL_NO_OUTPUT};
use rnp_cli_options::RnpCliOptions;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::runtime::Runtime;

mod rnp_cli_options;

#[cfg(not(tarpaulin_include))]
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let mut opts = RnpCliOptions::from_args();
    if opts.output_options.quiet_level < RNP_QUIET_LEVEL_NO_OUTPUT {
        println!("{} - {} - {}\n", RNP_NAME, RNP_AUTHOR, RNP_ABOUT);
    }

    opts.prepare_to_use();
    let runner_config = opts.to_ping_runner_config();

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let stop_event = Arc::new(ManualResetEvent::new(false));
        let rnp_exit_failure_reason = runner_config.result_processor_config.exit_failure_reason.clone();
        let mut runner = PingRunnerCore::new(runner_config, stop_event.clone());

        ctrlc::set_handler(move || {
            tracing::debug!("Ctrl+C received. Stopping all ping workers.");
            stop_event.set();
        })
        .expect("Error setting Ctrl-C handler");

        runner.run_warmup_pings().await;

        runner.start_running_normal_pings();
        runner.join().await;

        if let Some(rnp_exit_failure_reason) = rnp_exit_failure_reason {
            if rnp_exit_failure_reason.lock().unwrap().is_some() {
                return Err("Ping failed!".to_string());
            }
        }
        return Ok(());
    });

    // In order to have better control over the console output, we don't return the result from main function directly.
    if let Err(_) = result {
        std::process::exit(1);
    }
}
