use std::collections::BTreeMap;
use std::env;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use kpapp::cmd::cli::{cli, CONFIG_PATH_FLAG, HOMEDIR_FLAG, LOGLEVEL_FLAG};
use anyhow::{anyhow, Result};
use log::{debug, error, info, Log};
use tokio::sync::mpsc::Sender;
use kpapp::app::app::KPApp;
use kpapp::util::context::KPAppContext;
use kpcodec::util::alias::KPAVMediaType;
use kpcodec::util::encode_parameter::KPEncodeParameter;
use kpserver::server::server::KPServer;
use kpserver::util::service::KPService;
use crate::init::initialize;
use crate::util::event::{KPEventLoop, KPEventMessage};
use crate::util::server_event::KPServerEvent;
use kpserver::util::message::KPServerMessage;

mod init;
mod util;

const DEFAULT_CONFIG_FILENAME: &str = "kplayer.json";

#[tokio::main]
async fn main() {

    // parse command
    let command = cli();
    let matches = command.get_matches();

    let homedir = match matches.get_one::<String>(HOMEDIR_FLAG) {
        None => env::current_dir().expect("get current directory failed"),
        Some(p) => { PathBuf::from(p) }
    };
    if !homedir.exists() { error!("No such directory: {}", homedir.display()); }

    let config_path = match matches.get_one::<String>(CONFIG_PATH_FLAG) {
        None => homedir.join(DEFAULT_CONFIG_FILENAME),
        Some(p) => { PathBuf::from(p) }
    };
    if !config_path.exists() { error!("No such config file: {}", config_path.display()); }

    let log_level = matches.get_one::<String>(LOGLEVEL_FLAG).unwrap();

    // initialize
    initialize(Some(log_level.clone()));

    // create context
    let context = match KPAppContext::new(homedir.clone(), config_path.clone()) {
        Ok(c) => c,
        Err(err) => {
            error!("initialize app context failed. error: {}", err);
            return;
        }
    };
    debug!("load config file success. homedir: {}, config_path: {}", homedir.display(), config_path.display());

    // command exec
    match matches.subcommand() {
        Some(_) => {}
        None => {
            let event_loop = KPEventLoop::new();

            // start server
            let server_context_clone = context.clone();
            let server_sender_clone = event_loop.get_sender();
            tokio::spawn(async move {
                start_server(server_sender_clone.clone(), server_context_clone).await
            });

            // start transcode
            let transcode_context_clone = context.clone();
            let transcode_sender_clone = event_loop.get_sender();
            let transcode_broadcast_receiver = event_loop.subscribe();
            tokio::task::spawn_blocking(move || {
                futures::executor::block_on(async move {
                    start_transcode(transcode_sender_clone.clone(), transcode_broadcast_receiver, transcode_context_clone).await
                });
            });

            // event loop
            event_loop.event_loop().await;
        }
    }
}

async fn start_server(sender: Sender<KPEventMessage>, context: KPAppContext) {
    let notifier = KPServerEvent::new(sender.clone());
    let mut service = KPService::new(Arc::new(notifier));
    let output = context.config.output.clone();

    service.append(kpserver::util::config::KPConfig::rtmp_push {
        name: output.name.clone(),
        app_name: context.temporarily_server_app.clone(),
        stream_name: output.name.clone(),
        sink_url: output.path.clone(),
        timeout: Some(Duration::from_secs(10)),
        retry_interval: Some(Duration::from_secs(5)),
    });
    service.append(kpserver::util::config::KPConfig::rtmp {
        name: "core".to_string(),
        address: IpAddr::from_str("0.0.0.0").unwrap(),
        port: 1935,
        gop_number: 29,
    });

    let service_arc = Arc::new(service);

    // create server
    let mut server = KPServer::new(service_arc.clone());
    server.initialize().await;
    service_arc.wait().await;
}

async fn start_transcode(sender: Sender<KPEventMessage>, mut subscriber: tokio::sync::broadcast::Receiver<KPEventMessage>, mut context: KPAppContext) {
    let mut encode_parameter = BTreeMap::new();
    encode_parameter.insert(KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, KPEncodeParameter::default(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO));
    encode_parameter.insert(KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, KPEncodeParameter::default(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO));
    context.config.output.path = format!("rtmp://127.0.0.1:1935/{}/{}", context.temporarily_server_app, context.config.output.name);

    while let Ok(e) = subscriber.recv().await {
        if let KPEventMessage::server(server_msg) = e {
            if let KPServerMessage::RtmpStart { name, address, port } = server_msg {
                if name == "core" {
                    info!("core rtmp server start. name: {}", name);
                    break;
                }
            }
        }
    }

    let mut app = match KPApp::new(context, encode_parameter) {
        Ok(app) => {
            info!("create transcode app success");
            app
        }
        Err(err) => {
            error!("create transcode app failed. error: {}", err);
            return;
        }
    };
    match app.start().await {
        Ok(_) => {
            info!("transcode app exit success");
        }
        Err(err) => {
            error!("transcode app exit failed. error: {}", err);
        }
    };
}