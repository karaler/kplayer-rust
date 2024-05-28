pub mod error;
pub mod file;
pub mod jsonrpc;
pub mod rand;
pub mod time;
pub mod service_context;

pub fn default<T: Default>() -> T {
    Default::default()
}
