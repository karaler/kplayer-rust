use std::sync::Arc;
use log::info;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use streamhub::notify::Notifier;
use streamhub::StreamsHub;
use crate::notify::event_notifier::KPEventNotifier;
use crate::notify::log_notifier::KPLogNotifier;
use crate::notify::notifier::KPServerNotifier;
use crate::util::*;
use crate::util::message::KPServerMessage;

pub struct KPService {
    pub config: Vec<KPConfig>,
    pub stream_hub: Arc<Mutex<StreamsHub>>,
    pub notifier: Arc<dyn KPServerNotifier>,
}

impl KPService {
    pub fn new(notifier: Arc<dyn KPServerNotifier>) -> Self {
        let event_notifier = KPEventNotifier::new(notifier.clone());
        KPService {
            config: Vec::new(),
            stream_hub: Arc::new(Mutex::new(StreamsHub::new(Some(Arc::new(event_notifier))))),
            notifier,
        }
    }

    pub fn append(&mut self, cfg: KPConfig) {
        self.config.push(cfg)
    }

    pub async fn wait(&self) {
        let stream_hub = self.stream_hub.clone();
        let mut stream_hub = stream_hub.lock().await;
        stream_hub.run().await;
        info!("stream hub end...");
    }
}
