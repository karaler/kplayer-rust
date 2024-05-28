use std::sync::Arc;
use actix_web::web::service;
use crate::config::ServerSchema;
use crate::server::{KPGServer, ServerContext};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::{KPGConfigParseFailed, KPGServerMediaServerStartFailed};
use anyhow::Result;
use async_trait::async_trait;
use libkplayer::util::message::{KPMessage, KPMessageBus, MessageAction};
use libkplayer::util::message::MessageAction::*;
use log::{error, info};
use rtmp::rtmp::RtmpServer;
use streamhub::define::StreamHubEventMessage;
use streamhub::notify::Notifier;
use streamhub::StreamsHub;
use crate::util::service_context::{generate_service_context, ServiceContext};

struct KPMediaServerNotify {
    svc: ServiceContext,
}

impl KPMediaServerNotify {
    pub fn new(svc: &ServiceContext) -> Self {
        KPMediaServerNotify {
            svc: svc.clone()
        }
    }
}

#[async_trait]
impl Notifier for KPMediaServerNotify {
    async fn on_publish_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::Publish { identifier, info } = event {
            let msg = serde_json::to_string(info).unwrap();
            self.svc.message_sender.send(KPMessage {
                action: ServerAddPusher,
                message: msg,
            }).unwrap();
        }
    }

    async fn on_unpublish_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::UnPublish { identifier, info } = event {
            let msg = serde_json::to_string(info).unwrap();
            self.svc.message_sender.send(KPMessage {
                action: ServerRemovePusher,
                message: msg,
            }).unwrap();
        }
    }

    async fn on_play_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::Subscribe { identifier, info } = event {
            let msg = serde_json::to_string(info).unwrap();
            self.svc.message_sender.send(KPMessage {
                action: ServerAddPlayer,
                message: msg,
            }).unwrap();
        }
    }

    async fn on_stop_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::UnSubscribe { identifier, info } = event {
            let msg = serde_json::to_string(info).unwrap();
            self.svc.message_sender.send(KPMessage {
                action: ServerRemovePlayer,
                message: msg,
            }).unwrap();
        }
    }
}

pub struct KPMediaServer {
    svc: ServiceContext,
    name: String,
    server_context: Vec<ServerContext>,
    notifier: Arc<KPMediaServerNotify>,
    stream_hub: StreamsHub,
}

impl KPMediaServer {
    pub fn new<T: ToString>(svc: &ServiceContext, name: T, server_context: Vec<ServerContext>) -> Self {
        let notifier = Arc::new(KPMediaServerNotify::new(svc));
        let stream_hub = StreamsHub::new(Some(notifier.clone()));
        KPMediaServer {
            svc: svc.clone(),
            name: name.to_string(),
            stream_hub,
            server_context,
            notifier,
        }
    }

    async fn serve(&mut self) -> Result<()> {
        let address = format!("{}:{}", "0.0.0.0", 1935);
        let mut server = RtmpServer::new(address, self.stream_hub.get_hub_event_sender(), 1, None);

        let service_context = self.server_context.clone();
        tokio::spawn(async move {
            info!("media server listen success. context: {:?}",service_context);
            if let Err(err) = server.run().await {
                error!("media server listen failed. context: {:?}, error: {}", service_context,err)
            }
        });

        self.svc.message_sender.send(KPMessage { action: ServerListening, message: self.name.clone() })?;

        self.stream_hub.run().await;
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
    let svc = generate_service_context();
    env_logger::builder().filter_level(svc.log_level).init();

    let mut ms = KPMediaServer::new(&svc, "test", vec![ServerContext {
        schema: ServerSchema::Rtmp,
        name: "test".to_string(),
        address: "0.0.0.0".to_string(),
        port: 1935,
    }]);
    ms.serve().await.unwrap();
    info!("quit");
}
