use crate::config::ServerSchema;
use crate::util::error::KPGError;
use async_trait::async_trait;

pub mod http;
pub mod media;

#[derive(Clone, Debug)]
pub struct ServerContext {
    pub schema: ServerSchema,
    pub name: String,
    pub address: String,
    pub port: u16,
}

#[async_trait]
pub trait KPGServer: Send + Sync {
    async fn start(&mut self) -> Result<(), KPGError>;
    async fn stop(&mut self) -> Result<(), KPGError>;
    fn get_name(&self) -> String;
    fn get_context(&self, name: String) -> Option<ServerContext>;
}
