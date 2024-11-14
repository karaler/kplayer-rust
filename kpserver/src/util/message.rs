use std::net::IpAddr;
use std::time::Duration;
use strum_macros::Display;
use streamhub::define::{PublisherInfo, SubscriberInfo};
use streamhub::stream::StreamIdentifier;

#[derive(Clone, Display, Debug)]
pub enum KPServerMessage {
    RtmpStart {
        name: String,
        address: IpAddr,
        port: usize,
    },
    RtmpStop {
        name: String,
        error: Option<String>,
    },
    HttpflvStart {
        name: String,
    },
    HttpflvStop {
        name: String,
        error: Option<String>,
    },
    HlsStart {
        name: String,
    },
    HlsStop {
        name: String,
        error: Option<String>,
    },
    RtmpPullStart {
        name: String,
        source_url: String,
        retry_interval: Option<Duration>,
        retry_count: Option<usize>,
    },
    RtmpPullStop {
        name: String,
        source: String,
        error: Option<String>,
    },
    RtmpPushStart {
        name: String,
        sink_url: String,
    },
    RtmpPushStop {
        name: String,
        sink_url: String,
        error: Option<String>,
    },
    Publish {
        identifier: StreamIdentifier,
        info: PublisherInfo,
    },
    Unpublish {
        identifier: StreamIdentifier,
        info: PublisherInfo,
    },
    Subscribe {
        identifier: StreamIdentifier,
        info: SubscriberInfo,
    },
    Unsubscribe {
        identifier: StreamIdentifier,
        info: SubscriberInfo,
    },
    Unknown {
        name: String,
        error: String,
    },
}