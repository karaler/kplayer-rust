use log::info;
use streamhub::define::StreamHubEventMessage;
use streamhub::notify::Notifier;
use async_trait::async_trait;

pub struct KPLogNotifier {}

#[async_trait]
impl Notifier for KPLogNotifier {
    async fn on_publish_notify(&self, event: &StreamHubEventMessage) {
        info!("on publish notify. event: {:?}", event);
    }

    async fn on_unpublish_notify(&self, event: &StreamHubEventMessage) {
        info!("on unpublish notify. event: {:?}", event);
    }

    async fn on_play_notify(&self, event: &StreamHubEventMessage) {
        info!("on play notify. event: {:?}", event);
    }

    async fn on_stop_notify(&self, event: &StreamHubEventMessage) {
        info!("on stop notify. event: {:?}", event);
    }
}

impl KPLogNotifier {
    pub fn new() -> Self {
        KPLogNotifier {}
    }
}