use xiu::config::{Config, RtmpConfig};
use xiu::service::Service;
use anyhow::Result;
use log::info;
use tokio::signal::ctrl_c;
use crate::config::ServerSchema;
use crate::server::{KPGServer, ServerContext};
use crate::util::error::KPGError;

pub struct KPMediaServer {
    config: Config,
    service: Service,
}

impl KPMediaServer {
    pub fn new() -> Self {
        let cfg = Config::new(1935, 0, 0, 0, 0, "error".to_string());

        KPMediaServer {
            config: cfg.clone(),
            service: Service::new(cfg),
        }
    }

    pub async fn serve(&mut self) -> Result<()> {
        self.service.run().await?;
        Ok(())
    }
}

impl KPGServer for KPMediaServer {
    fn start(&mut self) -> std::result::Result<(), KPGError> {
        todo!()
    }

    fn stop(&mut self) -> std::result::Result<(), KPGError> {
        todo!()
    }

    fn get_schema(&self, schema: ServerSchema) -> Option<ServerContext> {
        todo!()
    }

    fn get_name(&self) -> String {
        todo!()
    }
}

#[tokio::test]
async fn test_server() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let mut ms = KPMediaServer::new();
    ms.serve().await.unwrap();

    ctrl_c().await.unwrap();
    info!("quit");
}