use log::info;
use tokio::sync::mpsc::{Receiver, Sender};
use streamhub::define::{PublisherInfo, SubscriberInfo};
use streamhub::stream::StreamIdentifier;

#[derive(Debug)]
pub enum KPEventMessage {
    server_publish {
        identifier: StreamIdentifier,
        info: PublisherInfo,
    },
    server_unpublish {
        identifier: StreamIdentifier,
        info: PublisherInfo,
    },
    server_subscribe {
        identifier: StreamIdentifier,
        info: SubscriberInfo,
    },
    server_unsubscribe {
        identifier: StreamIdentifier,
        info: SubscriberInfo,
    },
    server_start {},
    server_stop {
        error: anyhow::Error,
    },
    transcode_start {},
    transcode_stop {
        error: anyhow::Error,
    },
}

pub struct KPEventLoop {
    sender: Sender<KPEventMessage>,
    receiver: Receiver<KPEventMessage>,
}
impl KPEventLoop {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel::<KPEventMessage>(100);
        KPEventLoop { sender, receiver }
    }

    pub fn get_sender(&self) -> Sender<KPEventMessage> {
        self.sender.clone()
    }

    pub async fn event_loop(mut self) {
        while let Some(message) = self.receiver.recv().await {
            match message {
                KPEventMessage::server_publish { identifier, info } => {
                    info!("Publishing stream with identifier: {:?}, publisher info: {:?}", identifier, info);
                }
                KPEventMessage::server_unpublish { identifier, info } => {
                    info!("Unpublishing stream with identifier: {:?}, publisher info: {:?}", identifier, info);
                }
                KPEventMessage::server_subscribe { identifier, info } => {
                    info!("Playing stream with identifier: {:?}, subscriber info: {:?}", identifier, info);
                }
                KPEventMessage::server_unsubscribe { identifier, info } => {
                    info!("Stopping stream with identifier: {:?}, subscriber info: {:?}", identifier, info);
                }
                e => {
                    info!("Unhandled message: {:?}", e);
                }
            }
        }
    }
}