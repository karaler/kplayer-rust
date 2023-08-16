use crate::config::ServerSchema;
use crate::util::error::KPGError;

pub mod api;
pub mod media_server;
pub mod controller;

pub struct ServerContext {
    pub name: String,
    pub address: String,
    pub port: u32,
}

pub trait KPGServer: Send + Sync {
    fn start(&mut self) -> Result<(), KPGError>;
    fn stop(&mut self) -> Result<(), KPGError>;
    fn get_schema(&self, schema: ServerSchema) -> Option<ServerContext>;
    fn get_name(&self) -> String;
}