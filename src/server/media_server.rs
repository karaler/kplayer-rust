use std::collections::HashMap;
use std::fmt;
use enum_display::EnumDisplay;
use libkplayer::server::media_server::KPMediaServer;
use log::info;
use crate::config::{ServerSchema, ServerTokenType};
use crate::server::{KPGServer, ServerContext};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::*;

#[derive(Clone, Debug)]
pub struct KPGMediaServerContext {
    name: String,
    address: String,
    port: u32,
    schema: ServerSchema,
    token: ServerTokenType,
}

pub struct KPGMediaServer {
    name: String,
    media_server: KPMediaServer,
    contexts: HashMap<String, KPGMediaServerContext>,
    is_start: bool,
    is_closed: bool,
}

impl KPGMediaServer {
    pub fn new(name: String) -> KPGMediaServer {
        KPGMediaServer {
            name,
            media_server: KPMediaServer::new(),
            contexts: HashMap::new(),
            is_start: false,
            is_closed: false,
        }
    }

    pub fn add_rtmp(&mut self, name: String, address: String, port: u16, token: ServerTokenType) -> Result<(), KPGError> {
        let actual_port = match token.clone() {
            ServerTokenType::SingleToken { .. } => {
                return Err(KPGError::new_with_string(KPGServerMediaServerEnableSchemaFailed, format!("mismatch token type on the schema: rtmp, address: {}, port: {}, token: {}", address, port, token)));
            }
            ServerTokenType::CSToken { server, client } => {
                self.media_server.add_rtmp(name.clone(), address.clone(), port.clone() as u32, server, client).map_err(|err| {
                    KPGError::new_with_string(KPGServerMediaServerEnableSchemaFailed, format!("schema: rtmp, address: {}, port: {}, error: {}", address, port, err))
                })?
            }
        };

        let ctx = KPGMediaServerContext {
            name: name.clone(),
            address: address.clone(),
            port: actual_port,
            schema: ServerSchema::Rtmp,
            token: token.clone(),
        };
        info!("add media server group success. name: {}, context: {:?}", self.name, ctx.clone());
        self.contexts.insert(name.clone(), ctx);
        Ok(())
    }

    pub fn add_http(&mut self, name: String, address: String, port: u16, token: ServerTokenType) -> Result<(), KPGError> {
        let actual_port = match token.clone() {
            ServerTokenType::SingleToken { token } => {
                self.media_server.add_http(name.clone(), address.clone(), port.clone() as u32, token).map_err(|err| {
                    KPGError::new_with_string(KPGServerMediaServerEnableSchemaFailed, format!("schema: rtmp, address: {}, port: {}, error: {}", address, port, err))
                })?
            }
            ServerTokenType::CSToken { .. } => {
                return Err(KPGError::new_with_string(KPGServerMediaServerEnableSchemaFailed, format!("mismatch token type on the schema: http, address: {}, port: {}, token: {}", address, port, token)));
            }
        };

        let ctx = KPGMediaServerContext {
            name: name.clone(),
            address: address.clone(),
            port: actual_port,
            schema: ServerSchema::Http,
            token: token.clone(),
        };
        info!("add media server group success. name: {}, context: {:?}", self.name, ctx.clone());
        self.contexts.insert(name.clone(), ctx);
        Ok(())
    }
}

impl KPGServer for KPGMediaServer {
    fn start(&mut self) -> Result<(), KPGError> {
        assert!(!self.is_start);
        assert!(!self.is_closed);

        info!("start media server. name: {}, contexts: {:?}", self.name,self.contexts);
        self.is_start = true;

        self.media_server.start().map_err(|err| {
            KPGError::new_with_string(KPGServerMediaServerStartFailed, format!("media server start failed. error: {}", err))
        })
    }

    fn stop(&mut self) -> Result<(), KPGError> {
        assert!(self.is_start);
        assert!(!self.is_closed);

        self.is_closed = true;
        info!("stop media server. name: {}", self.name);

        self.media_server.stop().map_err(|err| {
            KPGError::new_with_string(KPGServerMediaServerStopFailed, format!("media server stop failed. error: {}", err))
        })
    }

    fn get_schema(&self, schema: ServerSchema) -> Option<ServerContext> {
        match self.contexts.iter().find(|(_, ctx)| {
            ctx.schema == schema
        }) {
            None => None,
            Some((name, ctx)) => {
                Some(ServerContext {
                    name: name.clone(),
                    address: ctx.address.clone(),
                    port: ctx.port.clone(),
                })
            }
        }
    }

    fn get_name(&self) -> String {
        return self.name.clone();
    }
}