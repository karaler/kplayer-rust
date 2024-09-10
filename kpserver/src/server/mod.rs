use std::sync::Arc;
use log::{debug, error, info};
use tokio::sync::{broadcast, Mutex};
use tokio::sync::broadcast::{Receiver, Sender};
use rtmp::rtmp::RtmpServer;
use streamhub::notify::Notifier;
use streamhub::StreamsHub;
use crate::notify::log_notifier::KPLogNotifier;
use crate::util::config::KPConfig;
use crate::server::status::KPServerMessage;
use crate::util::service::KPService;
use anyhow::{Result, anyhow};
use std::net::IpAddr;
use std::str::FromStr;

mod server;
mod status;