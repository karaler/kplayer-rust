pub mod error;
pub mod file;
pub mod jsonrpc;
pub mod rand;
pub mod time;

pub fn default<T: Default>() -> T {
    Default::default()
}
