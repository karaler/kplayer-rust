use std::io::Error;
use std::net::IpAddr;
use tokio::{join, select};
use hls::errors::HlsError;
use hls::remuxer::HlsRemuxer;
use crate::server::*;

pub struct KPServer {
    service: Arc<KPService>,
    stream_hub: Arc<Mutex<StreamsHub>>,
    message_sender: Sender<KPServerMessage>,
    message_receiver: Receiver<KPServerMessage>,
}

impl KPServer {
    pub fn new(service: Arc<KPService>, notifier: Arc<dyn Notifier>) -> Self {
        let (message_sender, message_receiver) = broadcast::channel::<KPServerMessage>(10);

        KPServer {
            service,
            stream_hub: Arc::new(Mutex::new(StreamsHub::new(Some(notifier)))),
            message_sender,
            message_receiver,
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        for cfg in self.service.get_config().iter() {
            match cfg {
                KPConfig::httpflv { port } => {
                    let producer = self.stream_hub.lock().await.get_hub_event_sender();
                    let msg_sender = self.message_sender.clone();
                    let port_clone = port.clone();

                    tokio::spawn(async move {
                        let error = match httpflv::server::run(producer, port_clone, None).await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("http-flv server start error: {}", err);
                                Some(err.to_string())
                            }
                        };
                        msg_sender.send(KPServerMessage::httpflv_stop { error }).unwrap();
                    });

                    debug!("http-flv server listen on {}", port);
                    self.message_sender.send(KPServerMessage::httpflv_start {})?;
                }
                KPConfig::hls { port } => {
                    let producer = self.stream_hub.lock().await.get_hub_event_sender();
                    let customer = self.stream_hub.lock().await.get_client_event_consumer();
                    let mut hls_remuxer = HlsRemuxer::new(customer, producer, false);
                    let port_clone = port.clone();

                    let msg_sender_remuxer = self.message_sender.clone();
                    tokio::spawn(async move {
                        let error = match hls_remuxer.run().await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("hls remuxer server start error: {}", err);
                                Some(err.to_string())
                            }
                        };
                        msg_sender_remuxer.send(KPServerMessage::hls_stop { error }).unwrap();
                    });

                    let msg_sender_server = self.message_sender.clone();
                    tokio::spawn(async move {
                        let error = match hls::server::run(port_clone, None).await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("hls server start error: {}", err);
                                Some(err.to_string())
                            }
                        };
                        msg_sender_server.send(KPServerMessage::hls_stop { error }).unwrap();
                    });

                    self.stream_hub.lock().await.set_hls_enabled(true);
                    debug!("hls server listen on {}", port);
                    self.message_sender.send(KPServerMessage::hls_start {})?;
                }
                KPConfig::rtmp { address, port, gop_number } => {
                    let producer = self.stream_hub.lock().await.get_hub_event_sender();
                    let bind_address = format!("{}:{}", address, port);
                    let mut rtmp_server = RtmpServer::new(bind_address.clone(), producer, gop_number.clone(), None);
                    let msg_sender = self.message_sender.clone();

                    tokio::spawn(async move {
                        let error = match rtmp_server.run().await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("rtmp server start error: {}", err);
                                Some(err.to_string())
                            }
                        };
                        msg_sender.send(KPServerMessage::rtmp_stop { error }).unwrap();
                    });

                    debug!("rtmp server listen on {}", bind_address);
                    self.message_sender.send(KPServerMessage::rtmp_start {})?;
                }
            }
        }
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        let stream_hub_locker = self.stream_hub.clone();
        tokio::spawn(async move {
            let mut stream_hub = stream_hub_locker.lock().await;
            stream_hub.run().await;
            info!("stream hub end...");
        });
        Ok(())
    }

    pub async fn wait(&mut self) -> Result<()> {
        loop {
            let msg = self.message_sender.subscribe().recv().await.unwrap();
            debug!("receiver message. msg: {}",msg);
            match msg {
                KPServerMessage::rtmp_stop { error } => {
                    if let Some(err) = error {
                        error!("server exit failed. error: {}",err);
                        return Err(anyhow!("server exit failed. error: {}",err));
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

#[tokio::test]
async fn test_server() {
    use crate::init::initialize;
    initialize();

    let log_notifier = KPLogNotifier::new();
    let mut service = KPService::new();
    service.append(KPConfig::rtmp {
        address: IpAddr::from_str("0.0.0.0").unwrap(),
        port: 1935,
        gop_number: 1,
    });
    service.append(KPConfig::httpflv { port: 8080 });
    service.append(KPConfig::hls { port: 8000 });

    let mut server = KPServer::new(Arc::new(service), Arc::new(log_notifier));
    server.initialize().await.unwrap();
    server.start().unwrap();

    server.wait().await.unwrap();
}