use log::info;
use async_trait::async_trait;
use crate::notify::notifier::KPServerNotifier;
use crate::util::message::KPServerMessage;

pub struct KPLogNotifier {}

#[async_trait]
impl KPServerNotifier for KPLogNotifier {
    async fn notify(&self, event: &KPServerMessage) {
        info!("server event: {:?}", event);
    }
}

impl KPLogNotifier {
    pub fn new() -> Self {
        KPLogNotifier {}
    }
}