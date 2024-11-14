use std::sync::Once;
use dotenv::dotenv;
use log::LevelFilter;
use kpcodec::init::set_ff_logger;

static INIT: Once = Once::new();

pub(crate) fn initialize(level_filter: Option<String>) {
    INIT.call_once(|| { dotenv().ok(); });

    let log_level = format!("error,kplayer={}", level_filter.unwrap_or("info".to_string()));
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // set codec core logger level
    set_ff_logger(Some(LevelFilter::Error));
}