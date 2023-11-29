use std::collections::HashMap;
use libkplayer::codec::component::media::{KPMedia, KPMediaResource};
use libkplayer::codec::playlist::{KPPlayList, PlayModel};
use libkplayer::util::kpcodec::avmedia_type::KPAVMediaType;
use log::{info, warn};
use crate::config::{ResourceType, ResourceTypeGroupPrimaryType, Root};
use crate::factory::KPGFactory;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::*;
use crate::util::file::read_directory_file;
use crate::util::time::KPDuration;

impl KPGFactory {
    pub(super) fn create_playlist(&mut self, cfg: &Root) -> Result<(), KPGError> {
        self.playlist = {
            let mut load_playlist = HashMap::new();
            for pl in cfg.playlist.iter() {
                let play_model = match pl.play_mode.clone() {
                    str if str == "list" => PlayModel::List,
                    str if str == "loop" => PlayModel::Loop,
                    str if str == "random" => PlayModel::Random,
                    _ => {
                        return Err(KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("invalid playlist play model. name: {}, play_model: {}", pl.name, pl.play_mode)));
                    }
                };
                let mut playlist = KPPlayList::new(pl.name.clone(), pl.start_point, play_model);
                for res in pl.resource.iter() {
                    match res {
                        ResourceType::ResourceSingle { path } => {
                            let mut media = KPMedia::new(KPMediaResource::SingleSource { path: path.clone() }, None, None, None);
                            let open_result = media.open();
                            if let Err(err) = open_result {
                                warn!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name,path,err)
                            } else {
                                let duration = media.get_duration();
                                playlist.add_media(media).map_err(|err| {
                                    KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name, path, err))
                                })?;
                                info!("add media to playlist success. playlist: {}, media: {}, duration: {:?}", pl.name,path, KPDuration::new(duration));
                            }
                        }
                        ResourceType::ResourceDirectory { directory, extension } => {
                            for file in read_directory_file(directory.clone(), extension.clone())? {
                                let mut media = KPMedia::new(KPMediaResource::SingleSource { path: file.clone() }, None, None, None);
                                if let Err(err) = media.open() {
                                    warn!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name,file,err)
                                } else {
                                    let duration = media.get_duration();
                                    playlist.add_media(media).map_err(|err| {
                                        KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name, file, err))
                                    })?;
                                    info!("add media to playlist success. playlist: {}, media: {}, duration: {:?}, from directory: {}", pl.name,file, KPDuration::new(duration), directory);
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

                            let mut media = KPMedia::new(KPMediaResource::SingleSource { path: path.clone() }, name.clone(), seek.clone(), end.clone());
                            media.set_expect_stream_index(expect_streams);
                            if let Err(err) = media.open() {
                                warn!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name,path,err)
                            } else {
                                let duration = media.get_duration();
                                playlist.add_media(media).map_err(|err| {
                                    KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("add media to playlist failed. playlist: {}, media: {}, error: {}", pl.name, path, err))
                                })?;
                                info!("add media to playlist success. playlist: {}, media: {}, duration: {:?}, seek: {:?}, end: {:?}", pl.name,path,KPDuration::new(duration),seek,end);
                            }
                        }
                        ResourceType::ResourceGroup { video_path, audio_path, primary_type, name, seek, end, stream } => {
                            let mut expect_streams = HashMap::new();
                            if let Some(get_stream) = stream {
                                if get_stream.video >= 0 { expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, get_stream.video as u32); }
                                if get_stream.audio >= 0 { expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, get_stream.audio as u32); }
                                if get_stream.subtitle >= 0 { expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_SUBTITLE, get_stream.subtitle as u32); }
                            }

                            let mut media = KPMedia::new(KPMediaResource::MixSource {
                                video_path: video_path.clone(),
                                audio_path: audio_path.clone(),
                                primary_media_type: {
                                    match primary_type {
                                        None => {
                                            KPAVMediaType::KPAVMEDIA_TYPE_UNKNOWN
                                        }
                                        Some(get_type) => {
                                            match get_type {
                                                ResourceTypeGroupPrimaryType::None => KPAVMediaType::KPAVMEDIA_TYPE_UNKNOWN,
                                                ResourceTypeGroupPrimaryType::Audio => KPAVMediaType::KPAVMEDIA_TYPE_AUDIO,
                                                ResourceTypeGroupPrimaryType::Video => KPAVMediaType::KPAVMEDIA_TYPE_VIDEO,
                                            }
                                        }
                                    }
                                },
                            }, name.clone(), seek.clone(), end.clone());
                            media.set_expect_stream_index(expect_streams);
                            if let Err(err) = media.open() {
                                warn!("add media to playlist failed. playlist: {}, media: {}, error: {}",pl.name, media.get_name(),err)
                            } else {
                                let duration = media.get_duration();
                                let media_path = media.get_path();
                                playlist.add_media(media).map_err(|err| {
                                    KPGError::new_with_string(KPGFactoryParseConfigFailed, format!("add media to playlist failed. playlist: {}, media: {:?}, error: {}", pl.name, media_path, err))
                                })?;
                                info!("add media to playlist success. playlist: {}, media: {:?}, duration: {:?}, seek: {:?}, end: {:?}", pl.name,media_path,KPDuration::new(duration),seek,end);
                            }
                        }
                    }
                }
                info!("create playlist success. playlist: {}, media total: {}", pl.name, playlist.get_media_list().len());
                load_playlist.insert(pl.name.clone(), playlist);
            }
            load_playlist
        };
        Ok(())
    }
}