use std::sync::Arc;
use log::{info, LevelFilter};
use tokio::sync::mpsc::{Receiver, Sender};
use kpapp::util::message::KPAppMessage;
use kpserver::util::message::KPServerMessage;

#[derive(Debug, Clone)]
pub enum KPEventMessage {
    server(KPServerMessage),
    transcode(KPAppMessage),
}

pub struct KPEventLoop {
    sender: Sender<KPEventMessage>,
    receiver: Receiver<KPEventMessage>,
    broadcast_sender: tokio::sync::broadcast::Sender<KPEventMessage>,
    broadcast_receiver: tokio::sync::broadcast::Receiver<KPEventMessage>,
}
impl KPEventLoop {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel::<KPEventMessage>(100);
        let (broadcast_sender, broadcast_receiver) = tokio::sync::broadcast::channel::<KPEventMessage>(100);
        KPEventLoop { sender, receiver, broadcast_sender, broadcast_receiver }
    }

    pub fn get_sender(&self) -> Sender<KPEventMessage> {
        self.sender.clone()
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<KPEventMessage> {
        self.broadcast_sender.subscribe()
    }

    pub async fn event_loop(mut self) {
        while let Some(message) = self.receiver.recv().await {
            // send to broadcast
            self.broadcast_sender.send(message.clone()).expect("broadcast message failed");

            // consume message
            match message {
                KPEventMessage::server(msg) => {
                    let mut default_msg_level = log::Level::Info;
                    match &msg {
                        KPServerMessage::RtmpStop { error, .. } => {
                            if error.is_some() { default_msg_level = log::Level::Error; }
                        }
                        KPServerMessage::HttpflvStop { error, .. } => {
                            if error.is_some() { default_msg_level = log::Level::Error; }
                        }
                        KPServerMessage::HlsStop { error, .. } => {
                            if error.is_some() { default_msg_level = log::Level::Error; }
                        }
                        KPServerMessage::RtmpPullStop { error, .. } => {
                            if error.is_some() { default_msg_level = log::Level::Error; }
                        }
                        KPServerMessage::RtmpPushStop { error, .. } => {
                            if error.is_some() { default_msg_level = log::Level::Error; }
                        }
                        KPServerMessage::Unknown { error, .. } => {}
                        _ => {}
                    }

                    log::log!(default_msg_level, "{:?}", msg);
                }
                KPEventMessage::transcode(msg) => {
                    info!("Received KPAppMessage: {:?}", msg);
                }
            }
        }
    }
}