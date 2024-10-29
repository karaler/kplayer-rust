use log::info;
use streamhub::define::StreamHubEventMessage;
use streamhub::notify::Notifier;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use kpcodec::util::codec_status::KPEncodeMode;
use crate::util::event::KPEventMessage;

pub struct KPServerNotifier {
    sender: Sender<KPEventMessage>,
}

#[async_trait]
impl Notifier for KPServerNotifier {
    async fn on_publish_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::Publish { identifier, info } = event.clone() {
            self.sender.send(KPEventMessage::server_publish { identifier, info }).await.expect("server notifier sender error");
        }
    }

    async fn on_unpublish_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::UnPublish { identifier, info } = event.clone() {
            self.sender.send(KPEventMessage::server_unpublish { identifier, info }).await.expect("server notifier sender error");
        }
    }

    async fn on_play_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::Subscribe { identifier, info } = event.clone() {
            self.sender.send(KPEventMessage::server_subscribe { identifier, info }).await.expect("server notifier sender error");
        }
    }

    async fn on_stop_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::UnSubscribe { identifier, info } = event.clone() {
            self.sender.send(KPEventMessage::server_unsubscribe { identifier, info }).await.expect("server notifier sender error");
        }
    }
}

impl KPServerNotifier {
    pub fn new(sender: Sender<KPEventMessage>) -> Self {
        KPServerNotifier {
            sender,
        }
    }
}