use crate::config::ServerSchema;
use crate::server::{KPGServer, ServerContext};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::{KPGConfigParseFailed, KPGServerMediaServerStartFailed};
use anyhow::Result;
use async_trait::async_trait;
use log::info;
use std::collections::HashMap;
use tokio::signal::ctrl_c;
use xiu::config::{Config, RtmpConfig};
use xiu::service::Service;

pub struct KPMediaServer {
    name: String,
    server_context: Vec<ServerContext>,
    config: Config,
    service: Service,
}

impl KPMediaServer {
    pub fn new<T: ToString>(name: T, server_context: Vec<ServerContext>) -> Self {
        let cfg = Config::new(1935, 0, 0, 0, 0, "error".to_string());

        KPMediaServer {
            name: name.to_string(),
            config: cfg.clone(),
            service: Service::new(cfg),
            server_context,
        }
    }

    async fn serve(&mut self) -> Result<()> {
        info!("media server listen success. context: {:?}", self.server_context);
        self.service.run().await?;
        Ok(())
    }
}

#[async_trait]
impl KPGServer for KPMediaServer {
    async fn start(&mut self) -> Result<(), KPGError> {
        self.serve().await.map_err(|err| {
            KPGError::new_with_string(KPGServerMediaServerStartFailed, err.to_string())
        })
    }

    async fn stop(&mut self) -> std::result::Result<(), KPGError> {
        Ok(())
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_context(&self, name: String) -> Option<ServerContext> {
        self.server_context
            .iter()
            .find(|&item| item.name == name)
            .cloned()
    }
}

#[tokio::test]
async fn test_server() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut ms = KPMediaServer::new("test", vec![]);
    ms.serve().await.unwrap();

    ctrl_c().await.unwrap();
    info!("quit");
}
