use std::ops::Deref;
use tokio::time::{interval, sleep};
use url::quirks::protocol;
use rtmp::session::errors::SessionError;
use streamhub::define::{StreamHubEvent, StreamHubEventSender};
use streamhub::stream::StreamIdentifier;
use crate::forward::*;

pub struct KPForward {
    service: Arc<KPService>,
}

impl KPForward {
    pub fn new(service: Arc<KPService>) -> Self {
        KPForward {
            service,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        for cfg in self.service.config.iter().cloned() {
            match cfg {
                KPConfig::rtmp_pull { name, source_url, app_name, stream_name, keep_alive, timeout, retry_interval } => {
                    let target_app_name = app_name;
                    let target_stream_name = stream_name;
                    let producer = self.service.stream_hub.lock().await.get_hub_event_sender();
                    let mut event_consumer = self.service.stream_hub.lock().await.get_client_event_consumer();

                    tokio::spawn(async move {
                        if !keep_alive {
                            loop {
                                let event = event_consumer.recv().await.unwrap();
                                if let BroadcastEvent::Subscribe {
                                    identifier: StreamIdentifier::Rtmp { app_name, stream_name, }, ..
                                } = event {
                                    let (source_address, source_app_name, source_stream_name) = KPForward::get_url_info(&source_url).unwrap();
                                    debug!("receive pull event. app_name: {}, stream_name: {}", app_name, stream_name);

                                    if source_app_name == app_name && source_stream_name == stream_name {
                                        info!("receive pull event, will open source url. source_url: {}, app_name: {}, stream_name: {}", source_address, app_name, stream_name);
                                        break;
                                    }
                                }
                            }
                        }

                        let mut retry_c = 1usize;
                        loop {
                            let producer = producer.clone();
                            // connect pull from source
                            if let Err(err) = KPForward::create_pull(producer, &source_url, target_app_name.clone(), target_stream_name.clone(), timeout).await {
                                error!("rtmp pull failed. source_url: {}, error: {}", source_url, err);
                            }

                            if let Some(d) = retry_interval {
                                info!("rtmp pull retry on {:?} after reconnect, retry count: {}", d, retry_c);
                                sleep(d).await;
                                retry_c += 1;
                            } else { break; }
                        }
                    });

                    self.service.stream_hub.lock().await.set_rtmp_pull_enabled(true);
                }
                KPConfig::rtmp_push { name, app_name, stream_name, sink_url, timeout, retry_interval } => {
                    let target_app_name = app_name;
                    let target_stream_name = stream_name;
                    let producer = self.service.stream_hub.lock().await.get_hub_event_sender();
                    let mut event_consumer = self.service.stream_hub.lock().await.get_client_event_consumer();

                    tokio::spawn(async move {
                        // wait source stream
                        loop {
                            match event_consumer.recv().await.unwrap() {
                                BroadcastEvent::Publish { identifier } => {
                                    if let StreamIdentifier::Rtmp { app_name, stream_name } = identifier {
                                        if app_name == target_app_name && stream_name == target_stream_name {
                                            break;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }

                        KPForward::create_push(producer.clone(), sink_url.clone(), target_app_name, target_stream_name, timeout).await.unwrap();
                        let s = 1;
                    });

                    self.service.stream_hub.lock().await.set_rtmp_push_enabled(true);
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn create_pull(producer: StreamHubEventSender, source_url: &String, app_name: Option<String>, stream_name: Option<String>, timeout: Option<Duration>) -> Result<()> {
        let (source_address, source_app_name, source_stream_name) = KPForward::get_url_info(&source_url)?;

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
        let (sink_address, sink_app_name, sink_stream_name) = KPForward::get_url_info(&sink_url)?;

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

    fn get_url_info(source_url: &String) -> Result<(String, String, String)> {
        let url = Url::parse(source_url.as_str())?;
        if url.scheme() != KPProtocol::Rtmp.to_string() { return Err(anyhow!("can not support forward source protocol. protocol: {}", url.scheme())); };
        let address = {
            let host = match url.host() {
                None => { return Err(anyhow!("source host can not be empty")); }
                Some(h) => h,
            };
            let port = url.port().unwrap_or(1935);
            format!("{}:{}", host, port)
        };
        let (app_name, stream_name) = match url.path_segments() {
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
        Ok((address, app_name, stream_name))
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

    let mut forward = KPForward::new(service_arc.clone());
    forward.initialize().await.unwrap();

    let mut server = KPServer::new(service_arc.clone());
    server.initialize().await.unwrap();

    service_arc.wait().await.unwrap();
}

#[tokio::test]
async fn test_push() {
    use crate::init::initialize;
    initialize();

    let log_notifier = KPLogNotifier::new();
    let mut service = KPService::new(Arc::new(log_notifier));
    let producer = service.stream_hub.lock().await.get_hub_event_sender();

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

    let mut forward = KPForward::new(service_arc.clone());
    forward.initialize().await.unwrap();

    let mut server = KPServer::new(service_arc.clone());
    server.initialize().await.unwrap();

    service_arc.wait().await.unwrap();
}