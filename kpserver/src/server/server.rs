use std::env;
use std::future::Future;
use std::io::Error;
use std::net::IpAddr;
use std::time::Duration;
use tokio::{join, select};
use tokio::net::TcpStream;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::sleep;
use hls::errors::HlsError;
use hls::remuxer::HlsRemuxer;
use rtmp::session::client_session::{ClientSession, ClientSessionType};
use streamhub::define::{BroadcastEvent, StreamHubEventSender};
use streamhub::stream::StreamIdentifier;
use crate::server::*;
use crate::util::parse_url::get_url_info;

pub struct KPServer {
    service: Arc<KPService>,
}

impl KPServer {
    pub fn new(service: Arc<KPService>) -> Self {
        KPServer {
            service,
        }
    }

    pub async fn initialize(&mut self) {
        let stream_hub = self.service.stream_hub.clone();
        let notifier = self.service.notifier.clone();
        for cfg in self.service.config.iter().cloned() {
            match cfg {
                KPConfig::httpflv { name, port } => {
                    let producer = stream_hub.lock().await.get_hub_event_sender();
                    let notifier_sender = notifier.clone();
                    let port_clone = port.clone();
                    let name_clone = name.clone();

                    tokio::spawn(async move {
                        let error = match httpflv::server::run(producer, port_clone, None).await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("http-flv server start error: {}, name: {}", err, name_clone);
                                Some(err.to_string())
                            }
                        };
                        notifier_sender.notify(&KPServerMessage::HttpflvStop { name: name_clone, error }).await;
                    });

                    debug!("http-flv server listen on {}, name: {}", port, name);
                    notifier.notify(&KPServerMessage::HttpflvStart { name: name.clone() }).await;
                }
                KPConfig::hls { name, port } => {
                    let producer = stream_hub.lock().await.get_hub_event_sender();
                    let customer = stream_hub.lock().await.get_client_event_consumer();
                    let mut hls_remuxer = HlsRemuxer::new(customer, producer, false);
                    let port_clone = port.clone();

                    let msg_notifier_remuxer = notifier.clone();
                    let name_remuxer_clone = name.clone();
                    tokio::spawn(async move {
                        let error = match hls_remuxer.run().await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("hls remuxer server start name: {}, error: {}",name_remuxer_clone, err);
                                Some(err.to_string())
                            }
                        };
                        msg_notifier_remuxer.notify(&KPServerMessage::HlsStop { name: name_remuxer_clone, error }).await;
                    });
                    notifier.notify(&KPServerMessage::HlsStart { name: name.clone() }).await;

                    let msg_notifier_server = notifier.clone();
                    let name_server_clone = name.clone();
                    tokio::spawn(async move {
                        let error = match hls::server::run(port_clone, None).await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("hls server start name: {}, error: {}", name_server_clone, err);
                                Some(err.to_string())
                            }
                        };
                        msg_notifier_server.notify(&KPServerMessage::HlsStop { name: name_server_clone, error }).await;
                    });

                    stream_hub.lock().await.set_hls_enabled(true);
                    debug!("hls server listen on {},name: {}", port, name);
                    notifier.notify(&KPServerMessage::HlsStart { name: name.clone() }).await;
                }
                KPConfig::rtmp { name, address, port, gop_number } => {
                    let producer = stream_hub.lock().await.get_hub_event_sender();
                    let bind_address = format!("{}:{}", address, port);
                    let mut rtmp_server = RtmpServer::new(bind_address.clone(), producer, gop_number.clone(), None);
                    let msg_notifier = notifier.clone();
                    let name_clone = name.clone();

                    tokio::spawn(async move {
                        let error = match rtmp_server.run().await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("rtmp server start name: {}, error: {}", name_clone, err);
                                Some(err.to_string())
                            }
                        };
                        msg_notifier.notify(&KPServerMessage::RtmpStop { name: name_clone, error }).await;
                    });

                    debug!("rtmp server listen on {}, name: {}", bind_address, name);
                    notifier.notify(&KPServerMessage::RtmpStart { name, address, port }).await;
                }
                KPConfig::rtmp_pull { name, source_url, app_name, stream_name, keep_alive, timeout, retry_interval } => {
                    let target_app_name = app_name;
                    let target_stream_name = stream_name;
                    let producer = self.service.stream_hub.lock().await.get_hub_event_sender();
                    let mut event_consumer = self.service.stream_hub.lock().await.get_client_event_consumer();
                    let msg_notifier = notifier.clone();

                    tokio::spawn(async move {
                        if !keep_alive {
                            loop {
                                match event_consumer.recv().await {
                                    Ok(event) => {
                                        if let BroadcastEvent::Subscribe {
                                            identifier: StreamIdentifier::Rtmp { app_name, stream_name, }, ..
                                        } = event {
                                            let (source_address, source_app_name, source_stream_name) = get_url_info(&source_url).unwrap();
                                            debug!("receive pull event. app_name: {}, stream_name: {}", app_name, stream_name);

                                            if source_app_name == app_name && source_stream_name == stream_name {
                                                info!("receive pull event, will open source url. source_url: {}, app_name: {}, stream_name: {}", source_address, app_name, stream_name);
                                                break;
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        msg_notifier.notify(&KPServerMessage::Unknown { name: name.clone(), error: err.to_string() }).await;
                                        return;
                                    }
                                }
                            }
                        }

                        let mut retry_c = 1usize;
                        loop {
                            let producer = producer.clone();

                            msg_notifier.notify(&KPServerMessage::RtmpPullStart {
                                name: name.clone(),
                                source_url: source_url.clone(),
                                retry_interval: retry_interval.clone(),
                                retry_count: match retry_interval {
                                    None => None,
                                    Some(_) => Some(retry_c.clone()),
                                },
                            }).await;

                            // connect pull from source
                            if let Err(err) = KPServer::create_pull(producer, &source_url, target_app_name.clone(), target_stream_name.clone(), timeout).await {
                                error!("rtmp pull failed. source_url: {}, error: {}", source_url, err);
                                msg_notifier.notify(&KPServerMessage::RtmpPullStop {
                                    name: name.clone(),
                                    source: source_url.clone(),
                                    error: Some(err.to_string()),
                                }).await;
                            }

                            // reconnect
                            if let Some(d) = retry_interval {
                                info!("rtmp pull retry on {:?} after reconnect, retry count: {}", d, retry_c);
                                sleep(d.clone()).await;
                                retry_c += 1;
                            } else {
                                msg_notifier.notify(&KPServerMessage::RtmpPullStop {
                                    name: name.clone(),
                                    source: source_url.clone(),
                                    error: None,
                                }).await;
                                break;
                            }
                        }
                    });

                    self.service.stream_hub.lock().await.set_rtmp_pull_enabled(true);
                }
                KPConfig::rtmp_push { name, app_name, stream_name, sink_url, timeout, retry_interval } => {
                    let target_app_name = app_name;
                    let target_stream_name = stream_name;
                    let producer = self.service.stream_hub.lock().await.get_hub_event_sender();
                    let mut event_consumer = self.service.stream_hub.lock().await.get_client_event_consumer();
                    let msg_notifier = notifier.clone();

                    tokio::spawn(async move {
                        if let Some(t) = timeout.clone() {
                            if let Err(err) = tokio::time::timeout(t, async {
                                while let Ok(event) = event_consumer.recv().await {
                                    if let BroadcastEvent::Publish { identifier } = event {
                                        if let StreamIdentifier::Rtmp { app_name, stream_name } = identifier {
                                            if app_name == target_app_name && stream_name == target_stream_name {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }).await {
                                msg_notifier.notify(&KPServerMessage::RtmpPushStop {
                                    name: name.clone(),
                                    sink_url: sink_url.clone(),
                                    error: Some(format!("find source resource timeout, error: {}", err)),
                                }).await;
                            }
                        };

                        msg_notifier.notify(&KPServerMessage::RtmpPushStart {
                            name: name.clone(),
                            sink_url: sink_url.clone(),
                        }).await;

                        // create push
                        let error = match KPServer::create_push(producer.clone(), sink_url.clone(), target_app_name, target_stream_name, timeout).await {
                            Ok(_) => None,
                            Err(err) => Some(err.to_string()),
                        };

                        msg_notifier.notify(&KPServerMessage::RtmpPushStop {
                            name: name.clone(),
                            sink_url: sink_url.clone(),
                            error,
                        }).await;
                    });

                    self.service.stream_hub.lock().await.set_rtmp_push_enabled(true);
                }
            }
        }
    }

    async fn create_pull(producer: StreamHubEventSender, source_url: &String, app_name: Option<String>, stream_name: Option<String>, timeout: Option<Duration>) -> Result<()> {
        let (source_address, source_app_name, source_stream_name) = get_url_info(&source_url)?;

        let stream = TcpStream::connect(source_address.clone()).await?;
        debug!("connect source url connection. source_url: {}", source_url);

        let mut client_session = ClientSession::new(
            stream,
            ClientSessionType::Pull,
            source_address.clone(),
            source_app_name.clone(),
            source_stream_name.clone(),
            producer.clone(),
            0,
        );
        client_session.set_publish(app_name.unwrap_or(source_app_name), stream_name.unwrap_or(source_stream_name));

        // set timeout
        if let Some(t) = timeout {
            client_session.set_timeout(t);
            debug!("set client session timeout. timeout: {:?}",t);
        }
        match client_session.run().await {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!("client session failed. error: {}",err))
        }
    }

    async fn create_push(producer: StreamHubEventSender, sink_url: String, app_name: String, stream_name: String, timeout: Option<Duration>) -> Result<()> {
        let (sink_address, sink_app_name, sink_stream_name) = get_url_info(&sink_url)?;

        let stream = TcpStream::connect(sink_address.clone()).await?;
        debug!("connect sink url connection. source_url: {}", sink_url);

        let mut client_session = ClientSession::new(
            stream,
            ClientSessionType::Push,
            sink_address,
            sink_app_name.clone(),
            sink_stream_name.clone(),
            producer.clone(),
            0,
        );
        client_session.subscribe(app_name, stream_name);

        // set timeout
        if let Some(t) = timeout {
            client_session.set_timeout(t);
            debug!("set client session timeout. timeout: {:?}",t);
        }
        match client_session.run().await {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!("client session failed. error: {}",err))
        }
    }
}

#[tokio::test]
async fn test_server() {
    use crate::init::initialize;
    initialize();

    let log_notifier = KPLogNotifier::new();
    let mut service = KPService::new(Arc::new(log_notifier));
    service.append(KPConfig::rtmp {
        name: "test".to_string(),
        address: IpAddr::from_str("0.0.0.0").unwrap(),
        port: 1935,
        gop_number: 1,
    });
    service.append(KPConfig::httpflv {
        name: "test".to_string(),
        port: 8080,
    });
    service.append(KPConfig::hls {
        name: "test".to_string(),
        port: 8000,
    });
    let service_arc = Arc::new(service);

    let mut server = KPServer::new(service_arc.clone());
    server.initialize().await;
    service_arc.wait().await;
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
        stream_name: Some("test".to_string()),
        keep_alive: false,
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
    let mut server = KPServer::new(service_arc.clone());
    server.initialize().await;

    service_arc.wait().await;
}

#[tokio::test]
async fn test_push() {
    use crate::init::initialize;
    initialize();

    let log_notifier = KPLogNotifier::new();
    let mut service = KPService::new(Arc::new(log_notifier));

    service.append(KPConfig::rtmp_pull {
        name: "pull".to_string(),
        source_url: "rtmp://arch.bytelang.com:1935/live/test".to_string(),
        app_name: None,
        stream_name: Some("tmp".to_string()),
        keep_alive: true,
        timeout: None,
        retry_interval: None,
    });
    service.append(KPConfig::rtmp_push {
        name: "push".to_string(),
        app_name: "live".to_string(),
        stream_name: "tmp".to_string(),
        sink_url: "rtmp://arch.bytelang.com:1935/live/forward".to_string(),
        timeout: Some(Duration::from_secs(10)),
        retry_interval: Some(Duration::from_secs(3)),
    });
    service.append(KPConfig::rtmp {
        name: "main".to_string(),
        address: IpAddr::from_str("0.0.0.0").unwrap(),
        port: 1935,
        gop_number: 0,
    });

    let service_arc = Arc::new(service);
    let mut server = KPServer::new(service_arc.clone());
    server.initialize().await;
    service_arc.wait().await;
}