use std::sync::Once;
use dotenv::dotenv;

static INIT: Once = Once::new();

pub(crate) fn initialize() {
    INIT.call_once(|| { dotenv().ok(); });
    // let log_level = "rtmp=error,h264_decoder=error,xflv=error,tokio=info,wasmtime=error,cranelift_codegen=error,cranelift_wasm=error,trace";
    let log_level = "info";
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();
}