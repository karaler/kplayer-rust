#![allow(E0004)]

use std::collections::HashMap;
use std::os::unix::raw::ino_t;
use std::rc::Rc;
use std::sync::Arc;
use actix_web::{App, HttpServer, middleware, web};
use futures::{join, select, TryFutureExt, TryStreamExt};
use libkplayer::bindings::nan;
use libkplayer::codec::playlist::KPPlayList;
use log::info;
use crate::config::ServerSchema;
use crate::factory::KPGFactory;
use crate::server::{KPGServer, ServerContext};
use crate::server::controller::playlist::{get_playlist_list};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGAPIServerBindFailed;

const MAX_JSON_BODY: usize = 1024 * 1024;

pub struct KPGApi {
    name: String,
    address: String,
    port: u16,
}

impl KPGApi {
    pub fn new(name: String, address: String, port: u16) -> KPGApi {
        KPGApi { name, address, port }
    }

    pub fn run(&self) -> Result<(), KPGError> {
        actix_rt::System::new().block_on(async {
            let address = self.address.clone();
            let port = self.port.clone();
            let name = self.name.clone();

            let server = HttpServer::new(|| {
                App::new().wrap(middleware::Logger::default())
                    .app_data(web::JsonConfig::default().limit(MAX_JSON_BODY))
                    .route("/playlist", web::get().to(move || {
                        get_playlist_list()
                    }))
            }).bind((self.address.as_str(), self.port)).map_err(|err| {
                KPGError::new_with_string(KPGAPIServerBindFailed, format!("address: {}, port: {}, error: {}", address, port, err))
            })?.run();

            let wait_signal = async move {
                return match actix_rt::signal::ctrl_c().await {
                    Ok(_) => {
                        info!("receive [Ctrl-C] signal. please wait, cleaning up resources.");
                        actix_rt::System::current().stop();
                        Ok(())
                    }
                    Err(err) => {
                        Err(KPGError::new_with_string(KPGAPIServerBindFailed, format!("register signal failed. name: {}, error: {}", name, err)))
                    }
                };
            };
            actix_rt::spawn(wait_signal);

            server.await.map_err(|err| {
                KPGError::new_with_string(KPGAPIServerBindFailed, format!("register signal failed. name: {}, error: {}", self.name, err))
            })?;
            info!("api server shutdown success. address: {}, port: {}", self.address,self.port);
            Ok(())
        })
    }
}

impl KPGServer for KPGApi {
    fn start(&mut self) -> Result<(), KPGError> {
        self.run()?;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), KPGError> {
        Ok(())
    }

    fn get_schema(&self, schema: ServerSchema) -> Option<ServerContext> {
        todo!()
    }

    fn get_name(&self) -> String {
        todo!()
    }
}