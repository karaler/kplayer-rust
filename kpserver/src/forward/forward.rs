use std::env;
use std::net::IpAddr;
use std::str::{FromStr, Split};
use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use log::{debug, error, info};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::sync::broadcast::error::RecvError;
use rtmp::relay::pull_client::PullClient;
use rtmp::relay::push_client::PushClient;
use streamhub::define::BroadcastEvent;
use streamhub::notify::Notifier;
use streamhub::StreamsHub;
use url::{Host, ParseError, Url};
use rtmp::session::client_session::{ClientSession, ClientSessionType};
use crate::notify::log_notifier::KPLogNotifier;
use crate::server::server::KPServer;
use crate::util::config::KPConfig;
use crate::util::const_var::KPProtocol;
use crate::util::service::KPService;
use crate::util::status::KPServerMessage;

pub struct KPForward {
    service: Arc<KPService>,
}

impl KPForward {
    pub fn new(service: Arc<KPService>) -> Self {
        KPForward {
            service,
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        let stream_hub = self.service.stream_hub.clone();
        for cfg in self.service.config.iter().cloned() {
            match cfg {
                KPConfig::rtmp_pull { name, source_url, app_name, stream_name, keep_alive, timeout, retry_interval } => {
                    let message_sender = self.service.message_sender.clone();
                    let mut stream_hub_guard = stream_hub.lock().await;
                    let producer = stream_hub_guard.get_hub_event_sender();
                    let name_clone = name.clone();
                    let msg_sender = message_sender.clone();
                    let source_clone = source_url.clone();

                    let message_sender_clone = message_sender.clone();
                    let source_c = source_clone.clone();
                    let deal = || async move {
                        let url = Url::parse(source_c.clone().as_str())?;
                        if url.scheme() != KPProtocol::Rtmp.to_string() { return Err(anyhow!("can not support forward source protocol. protocol: {}", url.scheme())); };
                        let address = {
                            let host = match url.host() {
                                None => { return Err(anyhow!("source host can not be empty")); }
                                Some(h) => h,
                            };
                            let port = url.port().unwrap_or(1935);
                            format!("{}:{}", host, port)
                        };
                        let (source_app_name, source_stream_name) = match url.path_segments() {
                            None => { return Err(anyhow!("source app name or stream name can not be empty")); }
                            Some(mut paths) => {
                                let app_name = match paths.nth(0) {
                                    None => { return Err(anyhow!("source app name or stream name can not be empty")); }
                                    Some(s) => s.to_string(),
                                };
                                let stream_name = match paths.nth(0) {
                                    None => { return Err(anyhow!("source app name or stream name can not be empty")); }
                                    Some(s) => s.to_string()
                                };
                                (app_name, stream_name)
                            }
                        };
                        let app_name = app_name.unwrap_or(source_app_name);
                        let stream_name = stream_name.unwrap_or(source_stream_name);

                        let stream = TcpStream::connect(address.clone()).await?;
                        debug!("connect source url connection. source_url: {}", source_clone);

                        let mut client_session = ClientSession::new(
                            stream,
                            ClientSessionType::Pull,
                            address.clone(),
                            app_name.clone(),
                            stream_name.clone(),
                            producer.clone(),
                            0,
                        );

                        // set timeout
                        if let Some(t) = timeout {
                            client_session.set_timeout(t);
                            debug!("set client session timeout. timeout: {:?}",t);
                        }

                        let app_name_clone = app_name.clone();
                        let stream_name_clone = stream_name.clone();
                        tokio::spawn(async move {
                            let error = match client_session.run().await {
                                Ok(_) => None,
                                Err(err) => {
                                    error!("rtmp pull source_url: {}, app_name: {}, stream_name: {}, error: {}", source_clone, app_name_clone, stream_name_clone, err);
                                    Some(err.to_string())
                                }
                            };
                            msg_sender.send(KPServerMessage::rtmp_pull_stop { name: name_clone, source: source_clone.clone(), error }).unwrap();
                        });

                        info!("rtmp pull open succeess. source_url: {}, app_name: {}, stream_name: {}", source_c, app_name, stream_name);
                        Ok(())
                    };
                    let error = match deal().await {
                        Ok(_) => None,
                        Err(e) => Some(e.to_string())
                    };

                    debug!("rtmp pull server quit. source_url: {}, error: {:?}", source_url, error);
                    message_sender_clone.send(KPServerMessage::rtmp_pull_start { name, source_url, error }).unwrap();
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_forward() {
    use crate::init::initialize;
    initialize();

    let log_notifier = KPLogNotifier::new();
    let mut service = KPService::new(Arc::new(log_notifier));
    service.append(KPConfig::rtmp_pull {
        name: "test".to_string(),
        source_url: env::var("SOURCE_URL").unwrap().to_string(),
        app_name: Some("live".to_string()),
        stream_name: Some("rtmp_pull".to_string()),
        keep_alive: true,
        timeout: Some(Duration::from_secs(2)),
        retry_interval: Some(Duration::from_secs(5)),
    });
    service.append(KPConfig::rtmp {
        name: "test".to_string(),
        address: IpAddr::from_str("0.0.0.0").unwrap(),
        port: 1935,
        gop_number: 1,
    });
    let service_arc = Arc::new(service);

    let mut forward = KPForward::new(service_arc.clone());
    forward.initialize().await.unwrap();

    let mut server = KPServer::new(service_arc.clone());
    server.initialize().await.unwrap();

    service_arc.wait().await.unwrap();
}