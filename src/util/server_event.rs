use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use kpserver::notify::notifier::KPServerNotifier;
use kpserver::util::message::KPServerMessage;
use crate::util::event::KPEventMessage;

#[derive(Debug, Clone)]
pub struct KPServerEvent {
    sender: Sender<KPEventMessage>,
}

#[async_trait]
impl KPServerNotifier for KPServerEvent {
    async fn notify(&self, event: &KPServerMessage) {
        self.sender.send(KPEventMessage::server(event.clone())).await.expect("send event message failed");
    }
}

impl KPServerEvent {
    pub fn new(sender: Sender<KPEventMessage>) -> Self {
        KPServerEvent {
            sender,
        }
    }
}