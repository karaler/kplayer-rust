use crate::config::ServerSchema;
use crate::server::{KPGServer, ServerContext};
use crate::util::error::KPGError;

pub struct KPGApi {}

impl KPGApi {
    pub fn new() -> KPGApi {
        KPGApi {}
    }
}

impl KPGServer for KPGApi {
    fn start(&mut self) -> Result<(), KPGError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), KPGError> {
        Ok(())
    }

    fn get_schema(&self, schema: ServerSchema) -> Option<ServerContext> {
        todo!()
    }

    fn get_name(&self) -> String {
        todo!()
    }
}