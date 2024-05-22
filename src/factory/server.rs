use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use libkplayer::plugin::plugin::KPPlugin;
use log::info;
use crate::config::{Root, ServerSchema, ServerType};
use crate::factory::KPGFactory;
use crate::server::api::KPGApi;
use crate::server::KPGServer;
use crate::server::media::KPMediaServer;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGFactoryParseConfigFailed;

impl KPGFactory {
    pub(super) fn create_server(&mut self, cfg: &Root) -> Result<(), KPGError> {
        self.server = {
            let mut servers: HashMap<String, Arc<Mutex<Box<dyn KPGServer>>>> = HashMap::new();
            for srv in cfg.server.iter() {
                match srv.target {
                    ServerType::None => {
                        return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("invalid server schema. schema: {:?}", srv.target)));
                    }
                    ServerType::Media => {
                        let mut media_server = KPMediaServer::new();
                        for g in srv.group.iter() {
                            match g.schema {
                                ServerSchema::Rtmp => {}
                                ServerSchema::Http => {}
                                _ => {
                                    return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("not support media server schema. schema: {:?}", g.schema)));
                                }
                            };
                        }
                        info!("create media server success. type: {:?}, server: {}",srv.target, srv.name);
                        servers.insert(srv.name.clone(), Arc::new(Mutex::new(Box::new(media_server))));
                    }
                    ServerType::Api => {
                        for g in srv.group.iter() {
                            match g.schema {
                                ServerSchema::Http => {
                                    servers.insert(srv.name.clone(), Arc::new(Mutex::new(Box::new(KPGApi::new(g.name.clone(), g.address.clone(), g.port.clone())))));
                                    info!("create api server success. type: {:?}, server: {}, address: {}, port: {}",srv.target, srv.name, g.address,g.port);
                                }
                                _ => {
                                    return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("not support api server schema. schema: {:?}", g.schema)));
                                }
                            };
                        }
                    }
                };
            };
            servers
        };
        Ok(())
    }
}