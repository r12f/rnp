use futures_intrusive::sync::ManualResetEvent;
use rnp::{RNP_ABOUT, RNP_AUTHOR, RNP_SERVER_NAME};
use rnp_server_cli_options::RnpServerCliOptions;
use std::error::Error;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::runtime::Runtime;

mod rnp_server_cli_options;

#[cfg(not(tarpaulin_include))]
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let mut opts = RnpServerCliOptions::from_args();
    println!("{} - {} - {}\n", RNP_SERVER_NAME, RNP_AUTHOR, RNP_ABOUT);

    opts.prepare_to_use();
    let config = opts.to_stub_server_config();

    let rt = Runtime::new().unwrap();
    let result: Result<(), Box<dyn Error + Send + Sync>> = rt.block_on(async move {
        let stop_event = Arc::new(ManualResetEvent::new(false));
        let server_started_event = Arc::new(ManualResetEvent::new(false));

        let stop_event_clone = stop_event.clone();
        ctrlc::set_handler(move || {
            tracing::debug!("Ctrl+C received. Stopping server.");
            stop_event_clone.set();
        })
        .expect("Error setting Ctrl-C handler");

        rnp::stub_server_factory::run(&config, stop_event, server_started_event).await??;

        return Ok(());
    });

    // In order to have better control over the console output, we don't return the result from main function directly.
    if let Err(e) = result {
        println!("Stub server failed to run: Error = {}", e);
        std::process::exit(1);
    }
}
