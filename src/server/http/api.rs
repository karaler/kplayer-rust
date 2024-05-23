#![allow(E0004)]

use actix_web::{App, HttpServer, middleware};
use log::info;
use crate::config::ServerSchema;
use crate::server::{KPGServer, ServerContext};
use crate::util::error::{KPGError, KPGErrorCode};
use crate::util::error::KPGErrorCode::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock, TryLockResult};
use actix_web::{get, post, HttpResponse, web};
use actix_web::dev::{HttpServiceFactory, Server};
use actix_web::test::read_body;
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::transform::KPTransform;
use libkplayer::get_global_console;
use libkplayer::util::console::*;
use libkplayer::util::console::KPConsolePrompt::{*};
use libkplayer::util::error::KPError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{GLOBAL_FACTORY, validate_and_respond_unprocessable_entity};
use crate::config::ResourceType;
use validator::{Validate, ValidationError};
use crate::server::http::instance::*;
use crate::server::http::playlist::*;
use anyhow::Result;
use async_trait::async_trait;


const MAX_JSON_BODY: usize = 1024 * 1024;

pub struct KPGHttp {
    name: String,
    server_context: Vec<ServerContext>,
}

impl KPGHttp {
    pub fn new(name: String, server_context: Vec<ServerContext>) -> Result<KPGHttp, KPGError> {
        for ctx in server_context.iter() {
            if ctx.schema != ServerSchema::Http {
                return Err(KPGError::new_with_string(KPGServerNotSupportSchema, format!("not support schema on http server. context: {:?}", ctx)));
            }
        }
        Ok(KPGHttp {
            name,
            server_context,
        })
    }

    async fn run(&self) -> Result<(), KPGError> {
        Ok(())
    }
}

#[async_trait]
impl KPGServer for KPGHttp {
    async fn start(&mut self) -> Result<(), KPGError> {
        for ctx in self.server_context.iter() {
            let server = HttpServer::new(|| {
                let mut app = App::new()
                    .wrap(middleware::Logger::default())
                    .app_data(web::JsonConfig::default().limit(MAX_JSON_BODY));
                // instance
                {
                    // list
                    app = app.service(get_instance_list);

                    // playlist
                    app = app.service(get_instance_playlist)
                        .service(get_instance_current)
                        .service(post_instance_prev)
                        .service(post_instance_skip)
                        .service(add_instance_media)
                        .service(remove_instance_media)
                        .service(move_instance_media)
                        .service(seek_instance_media)
                        .service(select_instance_media);

                    // basic
                    app = app.service(get_instance_info)
                        .service(get_instance_encode_parameter);

                    // plugin
                    app = app.service(get_instance_plugin)
                        .service(update_instance_plugin_argument);
                }
                app
            }).bind((ctx.address.as_str(), ctx.port)).map_err(|err| {
                KPGError::new_with_string(KPGAPIServerBindFailed, format!("context: {:?}, error: {}", ctx, err))
            })?.run();

            server.await.map_err(|err| {
                KPGError::new_with_string(KPGAPIServerStartFailed, format!("context: {:?}, error: {}", ctx, err))
            })?;

            info!("api server listen success. context: {:?}", ctx);
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), KPGError> {
        Ok(())
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_context(&self, name: String) -> Option<ServerContext> {
        self.server_context.iter().find(|&item| {
            item.name == name
        }).cloned()
    }
}