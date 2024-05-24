mod general;
mod instance;
mod output;
mod playlist;
mod scene;
mod server;
mod state;

use crate::config::env::get_homedir;
use crate::config::{OutputType, Playlist, ResourceType, Root, ServerSchema, ServerType};
use crate::factory::output::KPGOutput;
use crate::server::KPGServer;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::*;
use crate::util::file::read_directory_file;
use crate::util::rand::rand_string;
use crate::util::time::KPDuration;
use crate::util::{default, time};
use actix_web::web::to;
use bus::BusReader;
use libkplayer::bindings::{exit, AVCodecID_AV_CODEC_ID_H264};
use libkplayer::codec::component::filter::KPFilter;
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::playlist::{KPPlayList, PlayModel};
use libkplayer::codec::transform::KPTransform;
use libkplayer::plugin::plugin::KPPlugin;
use libkplayer::subscribe_message;
use libkplayer::util::console::{KPConsole, KPConsoleModule};
use libkplayer::util::error::KPError;
use libkplayer::util::kpcodec::avmedia_type::KPAVMediaType;
use libkplayer::util::kpcodec::kpencode_parameter::{
    KPEncodeParameterItem, KPEncodeParameterItemPreset, KPEncodeParameterItemProfile,
};
use libkplayer::util::message::{KPMessage, MessageAction};
use log::{debug, error, info, warn};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::os::unix::raw::ino_t;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Condvar};
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex, MutexGuard};
use tokio::time::sleep;

const PLUGIN_DIRECTORY: &str = "plugin/";
const PLUGIN_EXTENSION: &str = ".kpe";

const PROMPT_MAX_QUEUE_SIZE: usize = 5;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum ThreadType {
    Server,
    Instance,
    Output,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ThreadResult {
    pub thread_name: String,
    pub thread_type: ThreadType,
}

#[derive(Clone, Serialize)]
pub struct KPGFactoryInstance {
    #[serde(skip)]
    pub transform: Arc<Mutex<KPTransform>>,

    pub playlist: String,
    pub scene: Option<String>,
    pub server: String,
    pub is_launched: bool,
    pub created_at: u128,
}

pub struct KPGFactory {
    name: String,

    // parse from config
    playlist: HashMap<String, KPPlayList>,
    scene: HashMap<String, HashMap<String, KPPlugin>>,
    output: HashMap<String, Arc<tokio::sync::Mutex<KPGOutput>>>,
    server: HashMap<String, Arc<tokio::sync::Mutex<dyn KPGServer>>>,
    instance: HashMap<String, KPGFactoryInstance>,

    // runtime
    runtime: Runtime,

    // state
    create_time: u128,
    launch_time: u128,
    shutdown_time: u128,
}

impl Default for KPGFactory {
    fn default() -> Self {
        KPGFactory {
            name: Default::default(),
            playlist: Default::default(),
            scene: Default::default(),
            output: Default::default(),
            server: Default::default(),
            instance: Default::default(),
            runtime: Runtime::new().unwrap(),
            create_time: 0,
            launch_time: 0,
            shutdown_time: 0,
        }
    }
}

impl KPGFactory {
    pub fn new<T: ToString>(name: T) -> KPGFactory {
        KPGFactory {
            name: name.to_string(),
            ..default()
        }
    }

    pub async fn create(&mut self, cfg: Root) -> Result<(), KPGError> {
        self.create_playlist(&cfg).await?;
        self.create_scene(&cfg).await?;
        self.create_server(&cfg).await?;
        self.create_output(&cfg).await?;
        self.create_instance(&cfg).await?;

        self.create_time = KPDuration::current_mill_timestamp();
        Ok(())
    }

    pub async fn launch_server(&mut self, name: Option<&String>) -> Result<(), KPGError> {
        match name {
            None => {
                // launch all server
                let server = self.server.clone();

                self.runtime.spawn(async move {
                    for (name, svc) in server.iter() {
                        match svc.lock().await.start().await {
                            Ok(_) => {
                                info!("start server. name: {}", name)
                            }
                            Err(err) => {
                                todo!("notify state service quit")
                            }
                        };
                    }
                });
                Ok(())
            }
            Some(svc_name) => {
                let server = match self.server.get(svc_name) {
                    None => return Err(KPGError::new_with_string(KPGFactoryLaunchServerFailed, format!("name not found. name: {}", svc_name))),
                    Some(svc) => svc.clone(),
                };

                self.runtime.spawn(async move {
                    let mut svc = server.lock().await;
                    match svc.start().await {
                        Ok(_) => {
                            info!("start server. name: {}", svc.get_name())
                        }
                        Err(err) => {
                            todo!("notify state service quit")
                        }
                    }
                });
                Ok(())
            }
        }
    }

    pub async fn launch_instance(&mut self, name: Option<&String>) -> Result<(), KPGError> {
        match name {
            None => {
                // launch all instance
                let instances = self.instance.clone();
                for (name, instance) in instances {
                    tokio::spawn(async move {
                        let mut transform = instance.transform.lock().await;
                        info!("instance launch success. name: {}", name);
                        match transform.launch() {
                            Ok(_) => {
                                info!("launch instance shutdown success. name: {}", name);
                            }
                            Err(err) => {
                                panic!("{}", err)
                            }
                        }
                    });
                }
            }
            Some(instance_name) => {
                match self.instance.get(instance_name) {
                    None => return Err(KPGError::new_with_string(KPGFactoryLaunchInstanceFailed, format!("instance name not found. name: {}", instance_name))),
                    Some(instance) => {
                        let transform_arc = instance.clone().transform.clone();
                        tokio::spawn(async move {
                            let mut transform = transform_arc.lock().await;
                            info!("instance launch success. name: {}", transform.get_name());
                            match transform.launch(){
                                Ok(_) => {
                                    info!("launch instance shutdown success. name: {}", transform.get_name());
                                }
                                Err(_) => {
                                    todo!("notify error")
                                }
                            }
                        });
                    }
                }
            }
        };

        Ok(())
    }

    pub async fn launch_output(&mut self, name: Option<&String>) -> Result<(), KPGError> {
        match name {
            None => {
                let outputs = self.output.clone();
                self.runtime.spawn(async move {
                    for (name, output) in outputs {
                        let mut get_output = output.lock().await;
                        match get_output.serve().await {
                            Ok(_) => {
                                info!("launch output success. name: {}", get_output.get_name());
                            }
                            Err(_) => {
                                todo!("notify error")
                            }
                        }
                    }
                });
            }
            Some(_) => {}
        };
        Ok(())
    }
    pub async fn launch_message_bus(&mut self) -> Result<(), KPGError> {
        tokio::spawn(async move {
            for item in subscribe_message().iter() {}
        });
        Ok(())
    }

    pub async fn wait(&self) -> Result<(), KPGError> {
        sleep(Duration::from_secs(5000)).await;
        Ok(())
    }
}
