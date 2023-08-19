mod playlist;
mod scene;
mod server;
mod instance;
mod output;

use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver, sync_channel, SyncSender};
use std::thread::JoinHandle;
use bus::BusReader;
use libkplayer::bindings::{AVCodecID_AV_CODEC_ID_H264, exit};
use libkplayer::codec::component::filter::KPFilter;
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::playlist::{KPPlayList, PlayModel};
use libkplayer::codec::transform::KPTransform;
use libkplayer::plugin::plugin::KPPlugin;
use libkplayer::server::media_pusher::KPMediaPusher;
use libkplayer::subscribe_message;
use libkplayer::util::console::{KPConsole, KPConsoleModule};
use libkplayer::util::error::KPError;
use libkplayer::util::kpcodec::avmedia_type::KPAVMediaType;
use libkplayer::util::kpcodec::kpencode_parameter::{KPEncodeParameterItem, KPEncodeParameterItemPreset, KPEncodeParameterItemProfile};
use libkplayer::util::message::{KPMessage, MessageAction};
use log::{debug, error, info, warn};
use crate::config::{OutputType, Playlist, ResourceType, Root, ServerSchema, ServerType};
use crate::config::env::get_homedir;
use crate::server::api::{KPGApi};
use crate::server::KPGServer;
use crate::server::media_server::KPGMediaServer;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::*;
use crate::util::file::read_directory_file;
use crate::util::rand::rand_string;
use crate::util::time::KPDuration;

const PLUGIN_DIRECTORY: &str = "plugin/";
const PLUGIN_EXTENSION: &str = ".kpe";

const PROMPT_MAX_QUEUE_SIZE: usize = 5;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum ThreadType {
    Server,
    Instance,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ThreadResult {
    pub thread_name: String,
    pub thread_type: ThreadType,
}

pub struct KPGFactory {
    playlist: HashMap<String, KPPlayList>,
    scene: HashMap<String, HashMap<String, KPPlugin>>,
    output: HashMap<String, KPMediaPusher>,
    server: HashMap<String, Arc<Mutex<Box<dyn KPGServer>>>>,
    instance: HashMap<String, Arc<Mutex<KPTransform>>>,

    // thread
    exit_channel_sender: SyncSender<(ThreadResult, Result<(), KPGError>)>,
    exit_channel_receiver: Arc<Mutex<Receiver<(ThreadResult, Result<(), KPGError>)>>>,

    // state
    is_created: Arc<Mutex<bool>>,

    // console
    console: Arc<Mutex<KPConsole>>,
}

impl KPGFactory {
    pub fn new(console: Arc<Mutex<KPConsole>>) -> KPGFactory {
        let (exit_sender, exit_receiver) = sync_channel(1);
        return KPGFactory {
            playlist: Default::default(),
            scene: Default::default(),
            output: Default::default(),
            server: Default::default(),
            instance: Default::default(),
            exit_channel_sender: exit_sender,
            exit_channel_receiver: Arc::new(Mutex::new(exit_receiver)),
            is_created: Arc::new(Mutex::new(false)),
            console,
        };
    }

    pub fn set_console(&mut self, console: Arc<Mutex<KPConsole>>) {
        self.console = console
    }


    pub fn create(&mut self, cfg: Root) -> Result<(), KPGError> {
        self.create_playlist(&cfg)?;
        self.create_scene(&cfg)?;
        self.create_server(&cfg)?;
        self.create_instance(&cfg)?;
        self.create_output(&cfg)?;

        // set flag
        {
            let mut is_created = self.is_created.lock().unwrap();
            *is_created = true;
        }
        Ok(())
    }

    pub fn get_server_list(&self) -> Vec<String> {
        let mut server_name = Vec::new();
        for (name, _) in &self.server {
            server_name.push(name.clone());
        }

        server_name
    }

    pub fn launch_server(&mut self, name: &String) -> Result<ThreadResult, KPGError> {
        let server = self.server.get(name).unwrap();
        let server_clone = server.clone();
        let exit_sender_clone = self.exit_channel_sender.clone();

        let thread_result = ThreadResult {
            thread_name: name.clone(),
            thread_type: ThreadType::Server,
        };

        info!("launch server. thread result: {:?}", thread_result);
        let thread_result_clone = thread_result.clone();
        std::thread::spawn(move || {
            let mut get_server = server_clone.lock().unwrap();
            let result = get_server.start();
            exit_sender_clone.send((thread_result_clone, result.clone())).unwrap();
        });

        Ok(thread_result)
    }

    pub fn get_instance_list(&self) -> Vec<String> {
        let mut instance_name = Vec::new();
        for (name, _) in self.instance.iter() {
            instance_name.push(name.clone());
        }

        instance_name
    }

    pub fn get_instance(&self, name: &String) -> Option<Arc<Mutex<KPTransform>>> {
        match self.instance.get(name) {
            None => None,
            Some(transform) => {
                Some(transform.clone())
            }
        }
    }

    pub fn launch_instance(&mut self, name: &String) -> Result<ThreadResult, KPGError> {
        let transform = self.instance.get(name).unwrap();
        let transform_clone = transform.clone();
        let exit_sender_clone = self.exit_channel_sender.clone();

        let thread_result = ThreadResult {
            thread_name: name.clone(),
            thread_type: ThreadType::Instance,
        };
        info!("launch instance. thread result: {:?}", thread_result);

        // create prompt passer
        let thread_result_clone = thread_result.clone();
        let console_clone = self.console.clone();
        std::thread::spawn(move || {
            let mut get_transform = transform_clone.lock().unwrap();
            let name = get_transform.get_name();

            // register prompt receiver
            {
                let console_receiver = console_clone.lock().unwrap().register(KPConsoleModule::Instance, name.clone());
                get_transform.set_console_receiver(console_receiver);
            }

            // launch
            let result = get_transform.launch(None);
            let exit_result = match result {
                Ok(_) => {
                    Ok(())
                }
                Err(err) => {
                    Err(KPGError::new_with_string(KPGInstanceLaunchFailed, format!("instance launch failed. name: {}, error: {}", name, err)))
                }
            };
            exit_sender_clone.send((thread_result_clone, exit_result)).unwrap();
        });

        Ok(thread_result)
    }

    pub fn get_playlist(&self) -> HashMap<String, KPPlayList> {
        self.playlist.clone()
    }

    pub fn launch_message_bus(&mut self) {
        let exit_sender_clone = self.exit_channel_sender.clone();
        let thread_result = ThreadResult {
            thread_name: "message_bus".to_string(),
            thread_type: ThreadType::Instance,
        };

        let self_is_created = self.is_created.clone();
        std::thread::spawn(move || {
            let mut is_created = false;
            for item in subscribe_message().iter() {
                if !is_created {
                    is_created = self_is_created.lock().unwrap().clone();
                }
                if is_created {
                    info!("receive subscribe message. action: {}, message: {}", item.action,item.message);
                } else {
                    debug!("receive subscribe message. action: {}, message: {}", item.action,item.message);
                }
            }
            exit_sender_clone.send((thread_result, Ok(()))).unwrap();
        });
    }

    pub fn get_exit_receiver(&self) -> Arc<Mutex<Receiver<(ThreadResult, Result<(), KPGError>)>>> {
        self.exit_channel_receiver.clone()
    }

    pub fn read_plugin_content(plugin_name: &String) -> Result<Vec<u8>, KPGError> {
        let mut data = Vec::new();

        let mut file_path = get_homedir();
        file_path.push(PathBuf::from(PLUGIN_DIRECTORY));
        file_path.push(format!("{}{}", plugin_name, PLUGIN_EXTENSION));
        let mut fs = File::open(Path::new(file_path.to_str().unwrap())).map_err(|err| {
            KPGError::new_with_string(KPGFactoryOpenPluginFailed, format!("open plugin file failed. path: {}, error: {}", file_path.to_str().unwrap(), err))
        })?;
        fs.read_to_end(&mut data).map_err(|err| {
            KPGError::new_with_string(KPGFactoryOpenPluginFailed, format!("read plugin file failed. path: {}, error: {}", file_path.to_str().unwrap(), err))
        })?;

        Ok(data)
    }

    pub fn get_instance_source(name: String, port: u32) -> String {
        format!("rtmp://127.0.0.1:{}/live/{}", port, name)
    }

    pub fn get_instance_cache_path(name: &String) -> String {
        format!("cache/{}", name)
    }
}