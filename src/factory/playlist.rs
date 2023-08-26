use std::collections::HashMap;
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::playlist::{KPPlayList, PlayModel};
use libkplayer::util::kpcodec::avmedia_type::KPAVMediaType;
use log::{info, warn};
use crate::config::{ResourceType, Root};
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
                            let mut media = KPMedia::new(path.clone(), None, None, None);
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
                                let mut media = KPMedia::new(file.clone(), None, None, None);
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

                            let mut media = KPMedia::new(path.clone(), name.clone(), seek.clone(), end.clone());
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