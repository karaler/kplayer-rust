use crate::config::{Root, ServerSchema};
use crate::factory::{KPGFactory, KPGFactoryInstance};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGFactoryParseConfigFailed;
use crate::util::time::KPDuration;
use libkplayer::codec::transform::KPTransform;
use libkplayer::plugin::plugin::KPPlugin;
use libkplayer::util::kpcodec::kpencode_parameter::{
    KPEncodeParameterItem, KPEncodeParameterItemPreset, KPEncodeParameterItemProfile,
};
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

impl KPGFactory {
    pub(super) async fn create_instance(&mut self, cfg: &Root) -> Result<(), KPGError> {
        self.instance = {
            let mut instances = HashMap::new();
            for ins in cfg.instance.iter() {
                let instance_encode = ins.encode.clone();
                let mut encode_parameters = KPEncodeParameterItem::default();
                for mut encode_parameter in &mut encode_parameters {
                    match &mut encode_parameter {
                        KPEncodeParameterItem::Video {
                            ref mut fps,
                            ref mut width,
                            ref mut height,
                            ..
                        } => {
                            *fps = instance_encode.video.fps;
                            *width = instance_encode.video.width;
                            *height = instance_encode.video.height;
                        }
                        KPEncodeParameterItem::Audio {
                            ref mut channel_layout,
                            ref mut channels,
                            ref mut sample_rate,
                            ..
                        } => {
                            *channel_layout = instance_encode.audio.channel_layout;
                            *channels = instance_encode.audio.channels;
                            *sample_rate = instance_encode.audio.sample_rate;
                        }
                        KPEncodeParameterItem::General {
                            ref mut max_bit_rate,
                            ref mut avg_quality,
                            ref mut profile,
                            ref mut preset,
                            ref mut gop_uint,
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
                                            format!(
                                                "invalid encode profile. instance: {}, profile: {}",
                                                ins.name, instance_encode.profile
                                            ),
                                        ));
                                    }
                                }
                            };
                            *preset = {
                                match instance_encode.preset.clone() {
                                    str if str == String::from("veryfast") => {
                                        KPEncodeParameterItemPreset::VeryFast
                                    }
                                    _ => {
                                        return Err(KPGError::new_with_string(
                                            KPGFactoryParseConfigFailed,
                                            format!(
                                                "invalid encode preset. instance: {}, preset: {}",
                                                ins.name, instance_encode.preset
                                            ),
                                        ));
                                    }
                                }
                            };
                            *gop_uint = instance_encode.gop_uint;
                        }
                    }
                }

                let consistent_timestamp = {
                    match instance_encode.mode.clone() {
                        str if str == String::from("rtmp") => Some(true),
                        _ => Some(false),
                    }
                };
                let mut transform = KPTransform::new(
                    ins.name.clone(),
                    String::default(),
                    {
                        if ins.cache.on {
                            Some(KPGFactory::get_instance_cache_path(&ins.name))
                        } else {
                            None
                        }
                    },
                    Some(ins.enable_hardware),
                    encode_parameters,
                    consistent_timestamp,
                );

                // set playlist
                if !ins.playlist.is_empty() {
                    let mut get_playlist = {
                        if !self.playlist.contains_key(&ins.playlist) {
                            return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("couldn't find the playlist for the target configuration instance. instance: {}, playlist: {}", ins.name, ins.playlist)));
                        }
                        self.playlist.get(&ins.playlist).unwrap().clone()
                    };
                    get_playlist.init().map_err(|err| {
                        KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("couldn't initialize the playlist for the target configuration instance. instance: {}, playlist: {}, error: {}", ins.name, ins.playlist, err))
                    })?;
                    transform.set_playlist(get_playlist);
                }

                // set scene
                if let Some(scene) = &ins.scene {
                    let get_scene = {
                        if !self.scene.contains_key(scene) {
                            return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("couldn't find the scene for the target configuration instance. instance: {}, scene: {}", ins.name, scene)));
                        }
                        self.scene.get(scene).unwrap()
                    };

                    let mut plugin_group = HashMap::new();
                    for (name, scene_item) in get_scene {
                        let mut scene_plugin_item = scene_item.clone();
                        scene_plugin_item.load(Some(ins.name.clone()), Some(name.clone())).map_err(|err| {
                            KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("load scene plugin item failed. instance: {}, scene: {} plugin: {}, error: {}", ins.name, scene, name, err))
                        })?;

                        scene_plugin_item.open().map_err(|err| {
                            KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("open scene plugin item failed. instance: {}, scene: {} plugin: {}, error: {}", ins.name, scene, name, err))
                        })?;
                        plugin_group.insert(name.clone(), scene_plugin_item);
                    }
                    transform
                        .set_custom_plugin_group(plugin_group)
                        .map_err(|err| {
                            KPGError::new_with_string(
                                KPGFactoryParseConfigFailed,
                                format!(
                                    "can not set scene. instance: {}, scene: {}, error: {}",
                                    ins.name, scene, err
                                ),
                            )
                        })?;
                }

                // set server
                if !ins.server.is_empty() {
                    transform
                        .set_output_url(KPGFactory::get_instance_source(ins.name.clone(), 1935))
                }

                info!(
                    "create instance success. name: {}, playlist: {}, scene: {:?}, server: {}",
                    ins.name, ins.playlist, ins.scene, ins.server
                );
                instances.insert(
                    ins.name.clone(),
                    KPGFactoryInstance {
                        playlist: ins.playlist.clone(),
                        scene: ins.scene.clone(),
                        server: ins.server.clone(),
                        is_launched: false,
                        created_at: KPDuration::current_mill_timestamp(),
                        transform: Arc::new(Mutex::new(transform)),
                    },
                );
            }

            instances
        };
        Ok(())
    }
}
