use futures_intrusive::sync::ManualResetEvent;
use rnp::{RnpCore, RNP_ABOUT, RNP_AUTHOR, RNP_NAME};
use rnp_cli_options::RnpCliOptions;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::runtime::Runtime;

mod rnp_cli_options;

fn main() {
    println!("{} - {} - {}\n", RNP_NAME, RNP_AUTHOR, RNP_ABOUT);

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let mut opts = RnpCliOptions::from_args();
    opts.prepare_to_use();
    let rnp_core_config = opts.to_rnp_core_config();

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let stop_event = Arc::new(ManualResetEvent::new(false));
        let mut rp = RnpCore::new(rnp_core_config, stop_event.clone());

        ctrlc::set_handler(move || {
            tracing::debug!("Ctrl+C received. Stopping all ping workers.");
            stop_event.set();
        })
        .expect("Error setting Ctrl-C handler");

        rp.run_warmup_pings().await;

        rp.start_running_normal_pings();
        rp.join().await;
    });
}
