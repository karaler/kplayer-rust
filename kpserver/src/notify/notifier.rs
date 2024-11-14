use async_trait::async_trait;
use streamhub::define::StreamHubEventMessage;
use crate::util::message::KPServerMessage;

#[async_trait]
pub trait KPServerNotifier: Sync + Send {
    async fn notify(&self, event: &KPServerMessage);
}