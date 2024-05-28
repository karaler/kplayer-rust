use crate::config::{Root, ServerSchema, ServerType};
use crate::factory::KPGFactory;
use crate::server::http::api::KPGHttp;
use crate::server::media::KPMediaServer;
use crate::server::{KPGServer, ServerContext};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGFactoryParseConfigFailed;
use crate::util::service_context::ServiceContext;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;

impl KPGFactory {
    pub(super) async fn create_server(&mut self, svc: &ServiceContext) -> Result<(), KPGError> {
        self.server = {
            let mut servers: HashMap<String, Arc<tokio::sync::Mutex<dyn KPGServer>>> =
                HashMap::new();
            for srv in svc.config.server.iter() {
                match srv.target {
                    ServerType::None => {
                        return Err(KPGError::new_with_string(
                            KPGFactoryParseConfigFailed,
                            format!("invalid server schema. schema: {:?}", srv.target),
                        ));
                    }
                    ServerType::Media => {
                        let mut contexts = Vec::new();

                        for g in srv.group.iter() {
                            match g.schema {
                                ServerSchema::Rtmp => contexts.push(ServerContext {
                                    schema: g.schema.clone(),
                                    name: g.name.clone(),
                                    address: g.address.clone(),
                                    port: g.port.clone(),
                                }),
                                ServerSchema::Http => contexts.push(ServerContext {
                                    schema: g.schema.clone(),
                                    name: g.name.clone(),
                                    address: g.address.clone(),
                                    port: g.port.clone(),
                                }),
                                _ => {
                                    return Err(KPGError::new_with_string(
                                        KPGFactoryParseConfigFailed,
                                        format!(
                                            "not support media server schema. schema: {:?}",
                                            g.schema
                                        ),
                                    ));
                                }
                            };
                        }

                        let media_server = KPMediaServer::new(svc, srv.name.clone(), contexts);
                        info!(
                            "create media server success. type: {:?}, server: {}",
                            srv.target, srv.name
                        );
                        servers.insert(
                            srv.name.clone(),
                            Arc::new(tokio::sync::Mutex::new(media_server)),
                        );
                    }
                    ServerType::Api => {
                        for g in srv.group.iter() {
                            match g.schema {
                                ServerSchema::Http => {
                                    let http = KPGHttp::new(
                                        g.name.clone(),
                                        vec![ServerContext {
                                            schema: ServerSchema::Http,
                                            name: g.name.clone(),
                                            address: g.address.clone(),
                                            port: g.port.clone(),
                                        }],
                                    )?;
                                    servers.insert(
                                        srv.name.clone(),
                                        Arc::new(tokio::sync::Mutex::new(http)),
                                    );
                                    info!("create api server success. type: {:?}, server: {}, address: {}, port: {}",srv.target, srv.name, g.address,g.port);
                                }
                                _ => {
                                    return Err(KPGError::new_with_string(
                                        KPGFactoryParseConfigFailed,
                                        format!(
                                            "not support api server schema. schema: {:?}",
                                            g.schema
                                        ),
                                    ));
                                }
                            };
                        }
                    }
                };
            }
            servers
        };
        Ok(())
    }
}
