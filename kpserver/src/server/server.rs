use std::io::Error;
use std::net::IpAddr;
use tokio::{join, select};
use hls::errors::HlsError;
use hls::remuxer::HlsRemuxer;
use crate::server::*;

pub struct KPServer {
    service: Arc<KPService>,
}

impl KPServer {
    pub fn new(service: Arc<KPService>) -> Self {
        KPServer {
            service,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let stream_hub = self.service.stream_hub.clone();
        let message_sender = self.service.message_sender.clone();
        for cfg in self.service.config.iter() {
            match cfg {
                KPConfig::httpflv { name, port } => {
                    let producer = stream_hub.lock().await.get_hub_event_sender();
                    let msg_sender = message_sender.clone();
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
                        msg_sender.send(KPServerMessage::httpflv_stop { name: name_clone, error }).unwrap();
                    });

                    debug!("http-flv server listen on {}, name: {}", port, name);
                    message_sender.send(KPServerMessage::httpflv_start { name: name.clone() })?;
                }
                KPConfig::hls { name, port } => {
                    let producer = stream_hub.lock().await.get_hub_event_sender();
                    let customer = stream_hub.lock().await.get_client_event_consumer();
                    let mut hls_remuxer = HlsRemuxer::new(customer, producer, false);
                    let port_clone = port.clone();

                    let msg_sender_remuxer = message_sender.clone();
                    let name_remuxer_clone = name.clone();
                    tokio::spawn(async move {
                        let error = match hls_remuxer.run().await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("hls remuxer server start name: {}, error: {}",name_remuxer_clone, err);
                                Some(err.to_string())
                            }
                        };
                        msg_sender_remuxer.send(KPServerMessage::hls_stop { name: name_remuxer_clone, error }).unwrap();
                    });

                    let msg_sender_server = message_sender.clone();
                    let name_server_clone = name.clone();
                    tokio::spawn(async move {
                        let error = match hls::server::run(port_clone, None).await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("hls server start name: {}, error: {}", name_server_clone, err);
                                Some(err.to_string())
                            }
                        };
                        msg_sender_server.send(KPServerMessage::hls_stop { name: name_server_clone, error }).unwrap();
                    });

                    stream_hub.lock().await.set_hls_enabled(true);
                    debug!("hls server listen on {},name: {}", port, name);
                    message_sender.send(KPServerMessage::hls_start { name: name.clone() })?;
                }
                KPConfig::rtmp { name, address, port, gop_number } => {
                    let producer = stream_hub.lock().await.get_hub_event_sender();
                    let bind_address = format!("{}:{}", address, port);
                    let mut rtmp_server = RtmpServer::new(bind_address.clone(), producer, gop_number.clone(), None);
                    let msg_sender = message_sender.clone();
                    let name_clone = name.clone();

                    tokio::spawn(async move {
                        let error = match rtmp_server.run().await {
                            Ok(_) => None,
                            Err(err) => {
                                error!("rtmp server start name: {}, error: {}", name_clone, err);
                                Some(err.to_string())
                            }
                        };
                        msg_sender.send(KPServerMessage::rtmp_stop { name: name_clone, error }).unwrap();
                    });

                    debug!("rtmp server listen on {}, name: {}", bind_address, name);
                    message_sender.send(KPServerMessage::rtmp_start { name: name.clone() })?;
                }
                _ => {}
            }
        }
        Ok(())
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
    server.initialize().await.unwrap();
    service_arc.wait().await.unwrap();
}