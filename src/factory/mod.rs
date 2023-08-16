use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread::JoinHandle;
use libkplayer::bindings::{AVCodecID_AV_CODEC_ID_H264, exit};
use libkplayer::codec::component::filter::KPFilter;
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::playlist::{KPPlayList, PlayModel};
use libkplayer::codec::transform::KPTransform;
use libkplayer::plugin::plugin::KPPlugin;
use libkplayer::server::media_pusher::KPMediaPusher;
use libkplayer::util::error::KPError;
use libkplayer::util::kpcodec::avmedia_type::KPAVMediaType;
use libkplayer::util::kpcodec::kpencode_parameter::{KPEncodeParameterItem, KPEncodeParameterItemPreset, KPEncodeParameterItemProfile};
use log::{info, warn};
use crate::config::{OutputType, Playlist, ResourceType, Root, ServerSchema, ServerType};
use crate::config::env::get_homedir;
use crate::server::api::{KPGApi};
use crate::server::KPGServer;
use crate::server::media_server::KPGMediaServer;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::*;
use crate::util::file::read_directory_file;
use crate::util::rand::rand_string;

const PLUGIN_DIRECTORY: &str = "plugin/";
const PLUGIN_EXTENSION: &str = ".kpe";

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum ThreadType {
    Server,
    Instance,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ThreadResult {
    thread_name: String,
    thread_type: ThreadType,
}

pub struct KPGFactory {
    playlist: HashMap<String, KPPlayList>,
    scene: HashMap<String, HashMap<String, KPPlugin>>,
    output: HashMap<String, KPMediaPusher>,
    server: HashMap<String, Arc<Mutex<Box<dyn KPGServer>>>>,
    instance: HashMap<String, Arc<Mutex<KPTransform>>>,

    // thread
    thread_result: HashMap<ThreadResult, Option<Result<(), KPGError>>>,
    exit_channel_sender: SyncSender<(ThreadResult, Result<(), KPGError>)>,
    exit_channel_receiver: Receiver<(ThreadResult, Result<(), KPGError>)>,
}

impl KPGFactory {
    pub fn create(cfg: Root) -> Result<KPGFactory, KPGError> {
        // load playlist
        let playlist = {
            let mut load_playlist = HashMap::new();
            for pl in cfg.playlist {
                let play_model = match pl.play_mode {
                    str if str == "list" => PlayModel::List,
                    str if str == "loop" => PlayModel::Loop,
                    str if str == "random" => PlayModel::Random,
                    _ => {
                        return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("invalid playlist play model. name: {}, play_model: {}", pl.name, pl.play_mode)));
                    }
                };
                let mut playlist = KPPlayList::new(pl.name.clone(), pl.start_point, play_model);
                for res in pl.resource {
                    match res {
                        ResourceType::ResourceSingle { path } => {
                            let mut media = KPMedia::new(path.clone(), None, None, None);
                            let open_result = media.open();
                            if let Err(err) = open_result {
                                warn!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name,path,err)
                            } else {
                                let duration = media.get_duration();
                                playlist.add_media(media).map_err(|err| {
                                    KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name, path, err))
                                })?;
                                info!("add media to playlist success. playlist: {}, media: {}, duration: {}", pl.name,path, duration);
                            }
                        }
                        ResourceType::ResourceDirectory { directory, extension } => {
                            for file in read_directory_file(directory.clone(), extension)? {
                                let mut media = KPMedia::new(file.clone(), None, None, None);
                                if let Err(err) = media.open() {
                                    warn!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name,file,err)
                                } else {
                                    let duration = media.get_duration();
                                    playlist.add_media(media).map_err(|err| {
                                        KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name, file, err))
                                    })?;
                                    info!("add media to playlist success. playlist: {}, media: {}, duration: {}, directory: {}", pl.name,file, duration, directory);
                                }
                            }
                        }
                        ResourceType::ResourceDetail { path, name, seek, end, stream } => {
                            let mut expect_streams = HashMap::new();
                            if let Some(get_stream) = stream {
                                if get_stream.video >= 0 { expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, get_stream.video as u32); }
                                if get_stream.audio >= 0 { expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, get_stream.audio as u32); }
                                if get_stream.subtitle >= 0 { expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_SUBTITLE, get_stream.subtitle as u32); }
                            }

                            let mut media = KPMedia::new(path.clone(), name.clone(), seek.clone(), end.clone());
                            media.set_expect_stream_index(expect_streams);
                            if let Err(err) = media.open() {
                                warn!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name,path,err)
                            } else {
                                playlist.add_media(media).map_err(|err| {
                                    KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name, path, err))
                                })?;
                                info!("add media to playlist success. playlist: {}, media: {}, seek: {:?}, end: {:?}", pl.name,path,seek,end);
                            }
                        }
                    }
                }
                info!("create playlist success. playlist: {}, media count: {}", pl.name, playlist.get_media_list().len());
                load_playlist.insert(pl.name, playlist);
            }
            load_playlist
        };

        // load scene
        let mut scene = {
            let mut scenes = HashMap::new();
            for s in cfg.scene {
                let mut group = HashMap::new();
                for item in s.group {
                    let mut plugin = KPPlugin::new(KPGFactory::read_plugin_content(&item.app)?);
                    group.insert(item.name, plugin);
                }
                info!("create scene success. scene: {}", s.name);
                scenes.insert(s.name, group);
            }
            scenes
        };

        // load server
        let mut server = {
            let mut servers: HashMap<String, Arc<Mutex<Box<dyn KPGServer>>>> = HashMap::new();
            for srv in cfg.server {
                match srv.target {
                    ServerType::None => {
                        return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("invalid server schema. schema: {}", srv.target)));
                    }
                    ServerType::Media => {
                        let mut media_server = KPGMediaServer::new(srv.name.clone());
                        for g in srv.group {
                            match g.schema {
                                ServerSchema::Rtmp => {
                                    media_server.add_rtmp(g.name, g.address, g.port, g.token)?;
                                }
                                ServerSchema::Http => {
                                    media_server.add_http(g.name, g.address, g.port, g.token)?;
                                }
                                _ => {}
                            };
                        }
                        info!("create server success. type: {}, server: {}",srv.target, srv.name);
                        servers.insert(srv.name, Arc::new(Mutex::new(Box::new(media_server))));
                    }
                    ServerType::Api => {
                        info!("create server success. type: {}, server: {}",srv.target, srv.name);
                        servers.insert(srv.name, Arc::new(Mutex::new(Box::new(KPGApi::new()))));
                    }
                };
            };
            servers
        };

        // load instance
        let mut instance = {
            let mut instances = HashMap::new();
            for ins in cfg.instance {
                let instance_encode = ins.encode;
                let mut encode_parameters = KPEncodeParameterItem::default();
                for mut encode_parameter in &mut encode_parameters {
                    match &mut encode_parameter {
                        KPEncodeParameterItem::Video { ref mut fps, ref mut width, ref mut height, .. } => {
                            *fps = instance_encode.video.fps;
                            *width = instance_encode.video.width;
                            *height = instance_encode.video.height;
                        }
                        KPEncodeParameterItem::Audio { ref mut channel_layout, ref mut channels, ref mut sample_rate, .. } => {
                            *channel_layout = instance_encode.audio.channel_layout;
                            *channels = instance_encode.audio.channels;
                            *sample_rate = instance_encode.audio.sample_rate;
                        }
                        KPEncodeParameterItem::General {
                            ref mut max_bit_rate, ref mut avg_quality, ref mut profile,
                            ref mut preset, ref mut gop_uint
                        } => {
                            *max_bit_rate = instance_encode.max_bit_rate;
                            *avg_quality = instance_encode.avg_quality;
                            *profile = {
                                match instance_encode.profile.clone() {
                                    str if str == String::from("high") => {
                                        KPEncodeParameterItemProfile::High
                                    }
                                    _ => {
                                        return Err(KPGError::new_with_string(
                                            KPGFactoryParseConfigFailed,
                                            format!("invalid encode profile. instance: {}, profile: {}", ins.name, instance_encode.profile)));
                                    }
                                }
                            };
                            *preset = {
                                match instance_encode.preset.clone() {
                                    str if str == String::from("veryfast") => {
                                        KPEncodeParameterItemPreset::VeryFast
                                    }
                                    _ => {
                                        return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("invalid encode preset. instance: {}, preset: {}", ins.name, instance_encode.preset)));
                                    }
                                }
                            };
                            *gop_uint = instance_encode.gop_uint;
                        }
                    }
                }

                let consistent_timestamp = {
                    match instance_encode.mode.clone() {
                        str if str == String::from("rtmp") => {
                            Some(true)
                        }
                        _ => {
                            Some(false)
                        }
                    }
                };
                let mut transform = KPTransform::new(ins.name.clone(), String::default(), {
                    if ins.cache.on {
                        Some(KPGFactory::get_instance_cache_path(&ins.name))
                    } else {
                        None
                    }
                }, encode_parameters, consistent_timestamp);

                // set parameters
                if !ins.playlist.is_empty() {
                    let mut get_playlist = {
                        if !playlist.contains_key(&ins.playlist) {
                            return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("couldn't find the playlist for the target configuration instance. instance: {}, playlist: {}", ins.name, ins.playlist)));
                        }
                        playlist.get(&ins.playlist).unwrap().clone()
                    };
                    get_playlist.init().map_err(|err| {
                        KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("couldn't initialize the playlist for the target configuration instance. instance: {}, playlist: {}, error: {}", ins.name, ins.playlist, err))
                    })?;
                    transform.set_playlist(get_playlist);
                }

                // set scene
                if !ins.scene.is_empty() {
                    let get_scene = {
                        if !scene.contains_key(&ins.scene) {
                            return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("couldn't find the scene for the target configuration instance. instance: {}, scene: {}", ins.name, ins.scene)));
                        }
                        scene.get(&ins.scene).unwrap()
                    };

                    let mut plugin_group = HashMap::new();
                    for (name, scene_item) in get_scene {
                        let mut scene_plugin_item = scene_item.clone();
                        scene_plugin_item.load().map_err(|err| {
                            KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("load scene plugin item failed. instance: {}, scene: {} plugin: {}, error: {}", ins.name, ins.scene, name, err))
                        })?;

                        // @TODO add config params
                        scene_plugin_item.open(HashMap::new()).map_err(|err| {
                            KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("open scene plugin item failed. instance: {}, scene: {} plugin: {}, error: {}", ins.name, ins.scene, name, err))
                        })?;
                        plugin_group.insert(name.clone(), scene_plugin_item);
                    }
                    transform.set_custom_plugin_group(plugin_group).map_err(|err| {
                        KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("can not set scene. instance: {}, scene: {}, error: {}", ins.name, ins.scene, err))
                    })?;
                }

                // set server
                if !ins.server.is_empty() {
                    let get_server = {
                        if !server.contains_key(&ins.server) {
                            return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("couldn't find the server for the target configuration instance. instance: {}, scene: {}", ins.name, ins.scene)));
                        }
                        server.get(&ins.server).unwrap()
                    };
                    let server_guard = get_server.lock().unwrap();
                    if let Some(ctx) = server_guard.get_schema(ServerSchema::Rtmp) {
                        info!("Using name {} as the instance for streaming server. instance: {}", ctx.name, ins.server);
                        transform.set_output_url(KPGFactory::get_instance_source(ins.name.clone(), ctx.port))
                    }
                }

                info!("create instance success. name: {}, playlist: {}, scene: {}, server: {}",ins.name,ins.playlist,ins.scene,ins.server);
                instances.insert(ins.name, Arc::new(Mutex::new(transform)));
            }

            instances
        };

        // load output
        let output = {
            let mut outputs = HashMap::new();
            for out in cfg.output {
                let mut media_pusher = KPMediaPusher::new(out.name.clone());
                for group in &out.group {
                    match group {
                        OutputType::OutputSingle { path } => {
                            media_pusher.add_pusher(rand_string(8), path.clone(), 0);
                        }
                        OutputType::OutputDetail {
                            name,
                            reconnect_internal,
                            path,
                        } => {
                            media_pusher.add_pusher(name, path.clone(), reconnect_internal.clone());
                        }
                    }
                }
                info!("create output success. name: {}, source: {}, output: {:?}",out.name,out.source,out.group);
                outputs.insert(out.name.clone(), media_pusher);
            }
            outputs
        };

        let (exit_sender, exit_receiver) = sync_channel(1);
        let factory = KPGFactory {
            playlist,
            scene,
            output,
            server,
            instance,
            thread_result: HashMap::new(),
            exit_channel_sender: exit_sender,
            exit_channel_receiver: exit_receiver,
        };
        Ok(factory)
    }

    pub fn launch_server(&mut self) {
        for (name, server) in &self.server {
            let server_clone = server.clone();
            let exit_sender_clone = self.exit_channel_sender.clone();

            let thread_result = ThreadResult {
                thread_name: name.clone(),
                thread_type: ThreadType::Server,
            };
            assert!(!self.thread_result.contains_key(&thread_result));
            self.thread_result.insert(thread_result.clone(), None);
            info!("launch server. thread result: {:?}", thread_result);

            std::thread::spawn(move || {
                let mut get_server = server_clone.lock().unwrap();
                let result = get_server.start();
                exit_sender_clone.send((thread_result, result.clone())).unwrap();
            });
        }
    }

    pub fn launch_instance(&mut self) {
        for (name, transform) in &self.instance {
            let transform_clone = transform.clone();
            let name_clone = name.clone();
            let exit_sender_clone = self.exit_channel_sender.clone();

            let thread_result = ThreadResult {
                thread_name: name.clone(),
                thread_type: ThreadType::Instance,
            };
            assert!(!self.thread_result.contains_key(&thread_result));
            self.thread_result.insert(thread_result.clone(), None);
            info!("launch server. thread result: {:?}", thread_result);

            std::thread::spawn(move || {
                let mut get_transform = transform_clone.lock().unwrap();
                let name = get_transform.get_name();
                let result = get_transform.launch(None);
                let exit_result = match result {
                    Ok(_) => {
                        Ok(())
                    }
                    Err(err) => {
                        Err(KPGError::new_with_string(KPGInstanceLaunchFailed, format!("instance launch failed. name: {}, error: {}", name, err)))
                    }
                };
                exit_sender_clone.send((thread_result, exit_result)).unwrap();
            });
        }
    }

    pub fn wait(&mut self) -> Result<ThreadResult, Result<(), KPGError>> {
        let (thread_result, result) = self.exit_channel_receiver.recv().unwrap();
        self.thread_result.insert(thread_result.clone(), Some(result.clone()));

        let wait_items: Vec<_> = self.thread_result.iter().filter(|(thread_result, item)| {
            thread_result.thread_type == ThreadType::Instance && item.is_none()
        }).collect();
        if wait_items.is_empty() {
            return Ok(thread_result);
        }

        Err(result.clone())
    }

    pub fn check_instance_survival(&self) -> HashMap<String, bool> {
        let mut instance_state = HashMap::new();

        for (key, instance) in &self.instance {
            match instance.try_lock() {
                Ok(_) => {
                    instance_state.insert(key.clone(), false);
                }
                Err(err) => {
                    instance_state.insert(key.clone(), true);
                }
            }
        }

        instance_state
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