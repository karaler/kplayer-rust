use std::sync::Once;
use dotenv::dotenv;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        dotenv().ok();
    });

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}