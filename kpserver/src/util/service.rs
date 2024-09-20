use std::sync::Arc;
use log::info;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use streamhub::notify::Notifier;
use streamhub::StreamsHub;
use crate::notify::log_notifier::KPLogNotifier;
use crate::util::*;
use crate::util::status::KPServerMessage;

pub struct KPService {
    pub config: Vec<KPConfig>,
    pub stream_hub: Arc<Mutex<StreamsHub>>,
    pub message_sender: Sender<KPServerMessage>,
    pub message_receiver: Receiver<KPServerMessage>,
}

impl Default for KPService {
    fn default() -> Self {
        let log_notifier = KPLogNotifier::new();
        let (message_sender, message_receiver) = broadcast::channel::<KPServerMessage>(10);
        KPService {
            stream_hub: Arc::new(Mutex::new(StreamsHub::new(Some(Arc::new(log_notifier))))),
            message_sender,
            message_receiver,
            config: vec![KPConfig::rtmp {
                name: "test".to_string(),
                address: IpAddr::from_str("0.0.0.0").unwrap(),
                port: 1935,
                gop_number: 1,
            }],
        }
    }
}

impl KPService {
    pub fn new(notifier: Arc<dyn Notifier>) -> Self {
        let (message_sender, message_receiver) = broadcast::channel::<KPServerMessage>(10);
        KPService {
            config: Vec::new(),
            stream_hub: Arc::new(Mutex::new(StreamsHub::new(Some(notifier)))),
            message_sender,
            message_receiver,
        }
    }

    pub fn append(&mut self, cfg: KPConfig) {
        self.config.push(cfg)
    }

    pub async fn wait(&self) -> anyhow::Result<()> {
        let stream_hub = self.stream_hub.clone();
        let mut stream_hub = stream_hub.lock().await;
        stream_hub.run().await;
        info!("stream hub end...");
        Ok(())
    }
}
