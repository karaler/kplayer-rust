use std::sync::{Arc};
use libkplayer::util::message::KPMessage;
use tokio::sync::broadcast::{Receiver, Sender};
use log::{error, info, Level, LevelFilter};
use serde::de::Unexpected::Option;
use tokio::sync::Mutex;
use crate::config::{parse_file, Root};
use crate::setup_log;
use crate::util::error::KPGError;

pub type ServiceContext = Arc<ServiceContextPacket>;

pub fn generate_service_context() -> ServiceContext {
    ServiceContextPacket::new().expect("generate service context failed")
}

pub struct ServiceContextPacket {
    pub config: Root,
    pub log_level: LevelFilter,
    pub message_sender: Sender<KPMessage>,
    pub message_receiver: Arc<Mutex<Receiver<KPMessage>>>,
}

impl ServiceContextPacket {
    fn new() -> Result<ServiceContext, KPGError> {
        let config = parse_file()?;
        let (tx, rx) = tokio::sync::broadcast::channel(50);
        let svc = ServiceContextPacket {
            config,
            log_level: LevelFilter::Debug,
            message_sender: tx,
            message_receiver: Arc::new(Mutex::new(rx)),
        };

        Ok(Arc::new(svc))
    }
}