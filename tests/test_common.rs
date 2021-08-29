use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        let _ = env_logger::builder().format_timestamp_micros().is_test(true).try_init();
    });
}
