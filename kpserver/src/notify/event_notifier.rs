use std::sync::Arc;
use streamhub::define::StreamHubEventMessage;
use streamhub::notify::Notifier;
use async_trait::async_trait;
use crate::notify::notifier::KPServerNotifier;
use crate::util::message::KPServerMessage;

pub struct KPEventNotifier {
    notifier: Arc<dyn KPServerNotifier>,
}

#[async_trait]
impl Notifier for KPEventNotifier {
    async fn on_publish_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::Publish { identifier, info } = event.clone() {
            self.notifier.notify(&KPServerMessage::Publish { identifier, info }).await;
        }
    }

    async fn on_unpublish_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::UnPublish { identifier, info } = event.clone() {
            self.notifier.notify(&KPServerMessage::Unpublish { identifier, info }).await;
        }
    }

    async fn on_play_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::Subscribe { identifier, info } = event.clone() {
            self.notifier.notify(&KPServerMessage::Subscribe { identifier, info }).await;
        }
    }

    async fn on_stop_notify(&self, event: &StreamHubEventMessage) {
        if let StreamHubEventMessage::UnSubscribe { identifier, info } = event.clone() {
            self.notifier.notify(&KPServerMessage::Unsubscribe { identifier, info }).await;
        }
    }
}

impl KPEventNotifier {
    pub fn new(notifier: Arc<dyn KPServerNotifier>) -> Self {
        KPEventNotifier { notifier }
    }
}