use std::any::Any;
use std::collections::HashMap;
use std::fmt::format;
use libkplayer::bindings::rand;
use log::{debug, error, warn};
use serde_json::{Error, from_slice, json, Value};
use crate::config::{Instance, Output, OutputType, Playlist, ResourceType, Root, Scene, Server, ServerGroup, ServerSchema, ServerTokenType, ServerType};
use crate::config::OutputType::OutputSingle;
use crate::config::ResourceType::{ResourceDirectory, ResourceSingle};
use crate::config::ServerTokenType::{CSToken, SingleToken};
use crate::config::version::ParseConfig;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGConfigParseFailed;
use crate::util::rand::rand_string;

#[derive(Default)]
pub struct Version300 {}

impl ParseConfig for Version300 {
    fn parse(&self, data: &Vec<u8>) -> Result<Root, KPGError> {
        let json_value: Value = from_slice(data).map_err(|err| {
            KPGError::new_with_string(KPGConfigParseFailed, format!("parse version 3.0.0 failed. error: {}", err.to_string()))
        })?;

        let mut cfg = Root::default();
        for (key, value) in json_value.as_object().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "invalid root json format"))? {
            match key.clone() {
                str if str == String::from("version") => {
                    cfg.version = value.to_string();
                }
                str if str == String::from("playlist") => {
                    for playlist_item in value.as_array().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "invalid playlist json format"))? {
                        let mut playlist = Playlist::default();
                        playlist.name = playlist_item.get("name").ok_or(KPGError::new_with_str(KPGConfigParseFailed, "playlist name can not be empty"))?
                            .as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "playlist name invalid json format"))?
                            .to_string();
                        playlist.enable_hardware = match playlist_item.get("enable_hardware") {
                            None => true,
                            Some(get_enable_hardware) => get_enable_hardware.as_bool().unwrap_or(true),
                        };
                        playlist.start_point = match playlist_item.get("start_point") {
                            None => 0,
                            Some(val) => {
                                val.as_i64().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "playlist start_point invalid json format"))? as usize
                            }
                        };
                        playlist.play_mode = match playlist_item.get("play_mode") {
                            None => String::from("list"),
                            Some(val) => {
                                val.as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "playlist play_mode invalid json format"))?.to_string()
                            }
                        };
                        playlist.universal_seek = match playlist_item.get("universal_seek") {
                            None => -1,
                            Some(val) => {
                                val.as_i64().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "playlist universal_seek invalid json format"))?
                            }
                        };
                        playlist.universal_end = match playlist_item.get("universal_end") {
                            None => -1,
                            Some(val) => {
                                val.as_i64().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "playlist universal_end invalid json format"))?
                            }
                        };
                        playlist.universal_reverse_seek = match playlist_item.get("universal_seek") {
                            None => -1,
                            Some(val) => {
                                val.as_i64().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "playlist universal_reverse_seek invalid json format"))?
                            }
                        };
                        playlist.universal_reverse_end = match playlist_item.get("universal_end") {
                            None => -1,
                            Some(val) => {
                                val.as_i64().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "playlist universal_reverse_end invalid json format"))?
                            }
                        };

                        let get_resource = playlist_item.get("resource").ok_or(KPGError::new_with_str(KPGConfigParseFailed, "invalid playlist resource json format"))?
                            .as_array().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "invalid playlist resource json format"))?;
                        for resource_item in get_resource {
                            let result = match resource_item {
                                Value::String(val) => {
                                    ResourceSingle { path: val.to_string() }
                                }
                                Value::Object(val) => {
                                    let mut resource = Default::default();
                                    if val.contains_key("type") {
                                        let get_type = val.get("type").unwrap().as_str().unwrap();
                                        match get_type {
                                            "detail" => {
                                                let mut content = HashMap::new();
                                                content.insert("ResourceDetail", val);
                                                resource = serde_json::from_value(json!(content)).map_err(|err| {
                                                    KPGError::new_with_string(KPGConfigParseFailed, format!("playlist resource invalid json format, error: {}", err))
                                                })?;
                                            }
                                            "directory" => {
                                                let mut content = HashMap::new();
                                                content.insert("ResourceDirectory", val);
                                                resource = serde_json::from_value(json!(content)).map_err(|err| {
                                                    KPGError::new_with_string(KPGConfigParseFailed, format!("playlist resource invalid json format, error: {}", err))
                                                })?;
                                            }
                                            "group" => {
                                                let mut content = HashMap::new();
                                                content.insert("ResourceGroup", val);
                                                resource = serde_json::from_value(json!(content)).map_err(|err| {
                                                    KPGError::new_with_string(KPGConfigParseFailed, format!("playlist resource invalid json format, error: {}", err))
                                                })?;
                                            }
                                            &_ => {
                                                error!("not support object attribute for resource type. type: {}", get_type);
                                            }
                                        }
                                    } else {
                                        error!("invalid object attribute for resource type");
                                    }

                                    resource
                                }
                                _ => {
                                    return Err(KPGError::new_with_string(KPGConfigParseFailed, format!("playlist resource invalid json format")));
                                }
                            };
                            playlist.resource.push(result);
                        }

                        cfg.playlist.push(playlist);
                    }
                }
                str if str == String::from("scene") => {
                    match serde_json::from_value::<Vec<Scene>>(json!(value)) {
                        Ok(get_scenes) => {
                            cfg.scene = get_scenes
                        }
                        Err(err) => {
                            return Err(KPGError::new_with_string(KPGConfigParseFailed, format!("scene invalid json format. error: {}", err)));
                        }
                    }
                }
                str if str == String::from("output") => {
                    for output_item in value.as_array().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "invalid output json format"))? {
                        let mut output = Output::default();
                        output.name = output_item.get("name").ok_or(KPGError::new_with_str(KPGConfigParseFailed, "output name can not be empty"))?
                            .as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "output name invalid json format"))?
                            .to_string();
                        output.source = output_item.get("source").ok_or(KPGError::new_with_str(KPGConfigParseFailed, "output source can not be empty"))?
                            .as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "output source invalid json format"))?
                            .to_string();
                        for group_item in output_item.get("group").ok_or(KPGError::new_with_str(KPGConfigParseFailed, "output group can not be empty"))?
                            .as_array().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "output group invalid json format"))? {
                            let group = match group_item {
                                Value::String(val) => {
                                    OutputSingle { path: val.to_string() }
                                }
                                Value::Object(val) => {
                                    let mut content = HashMap::new();
                                    content.insert("OutputDetail", val);
                                    serde_json::from_value(json!(content)).map_err(|err| {
                                        KPGError::new_with_string(KPGConfigParseFailed, format!("playlist output invalid json format, error: {}", err))
                                    })?
                                }
                                _ => {
                                    return Err(KPGError::new_with_string(KPGConfigParseFailed, format!("playlist output invalid json format")));
                                }
                            };
                            output.group.push(group);
                        };
                        cfg.output.push(output);
                    }
                }
                str if str == String::from("server") => {
                    for server_item in value.as_array().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "invalid server json format"))? {
                        let mut srv = Server::default();
                        srv.name = server_item.get("name").ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server name can not be empty"))?
                            .as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server name invalid json format"))?
                            .to_string();
                        srv.target = match server_item.get("target").ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server target can not be empty"))?
                            .as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server target invalid json format"))?
                            .to_string() {
                            str if str == "media" => {
                                ServerType::Media
                            }
                            str if str == "api" => {
                                ServerType::Api
                            }
                            _ => {
                                return Err(KPGError::new_with_string(KPGConfigParseFailed, format!("not support server target")));
                            }
                        };
                        for group_item in server_item.get("group").ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server name can not be empty"))?
                            .as_array().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server name invalid json format"))? {
                            let mut group = ServerGroup::default();
                            group.name = match group_item.get("name") {
                                None => { rand_string(8) }
                                Some(val) => {
                                    val.as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server group invalid json format"))?.to_string()
                                }
                            };
                            group.address = match group_item.get("address") {
                                None => String::from("127.0.0.1"),
                                Some(val) => {
                                    val.as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server group address invalid json format"))?.to_string()
                                }
                            };
                            group.schema = match group_item.get("schema") {
                                None => {
                                    return Err(KPGError::new_with_str(KPGConfigParseFailed, "server group schema can not be empty"));
                                }
                                Some(val) => {
                                    match val.as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server group address invalid json format"))? {
                                        str if str == "rtmp" => {
                                            ServerSchema::Rtmp
                                        }
                                        str if str == "http" => {
                                            ServerSchema::Http
                                        }
                                        _ => {
                                            return Err(KPGError::new_with_string(KPGConfigParseFailed, format!("server group not support schema. schema: {}", val)));
                                        }
                                    }
                                }
                            };
                            group.port = match group_item.get("port") {
                                None => {
                                    return Err(KPGError::new_with_str(KPGConfigParseFailed, "server group port can not be empty"));
                                }
                                Some(val) => {
                                    val.as_i64().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server group port invalid json format"))? as u16
                                }
                            };
                            group.token = match srv.target {
                                ServerType::Media => {
                                    match group_item.get("token") {
                                        None => {
                                            match group.schema {
                                                ServerSchema::Rtmp => {
                                                    CSToken {
                                                        server: String::default(),
                                                        client: String::default(),
                                                    }
                                                }
                                                ServerSchema::Http => {
                                                    SingleToken { token: String::default() }
                                                }
                                                _ => {
                                                    return Err(KPGError::new_with_str(KPGConfigParseFailed, "invalid server group schema"));
                                                }
                                            }
                                        }
                                        Some(token_val) => {
                                            match token_val {
                                                Value::String(_) => {
                                                    match group.schema {
                                                        ServerSchema::Rtmp => {
                                                            CSToken {
                                                                server: token_val.to_string(),
                                                                client: token_val.to_string(),
                                                            }
                                                        }
                                                        ServerSchema::Http => {
                                                            SingleToken { token: token_val.to_string() }
                                                        }
                                                        ServerSchema::None => {
                                                            return Err(KPGError::new_with_str(KPGConfigParseFailed, "invalid server group schema"));
                                                        }
                                                    }
                                                }
                                                Value::Object(_) => {
                                                    CSToken {
                                                        server: match token_val.get("server") {
                                                            None => { String::default() }
                                                            Some(val) => { val.as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server token server-side token invalid json format"))?.to_string() }
                                                        },
                                                        client: match token_val.get("client") {
                                                            None => { String::default() }
                                                            Some(val) => { val.as_str().ok_or(KPGError::new_with_str(KPGConfigParseFailed, "server token client-side token invalid json format"))?.to_string() }
                                                        },
                                                    }
                                                }
                                                _ => {
                                                    return Err(KPGError::new_with_string(KPGConfigParseFailed, format!("server token invalid json format")));
                                                }
                                            }
                                        }
                                    }
                                }
                                ServerType::Api => {
                                    match group_item.get("token") {
                                        None => {
                                            SingleToken { token: String::default() }
                                        }
                                        Some(token_val) => {
                                            SingleToken { token: token_val.to_string() }
                                        }
                                    }
                                }
                                _ => {
                                    return Err(KPGError::new_with_string(KPGConfigParseFailed, format!("server type invalid")));
                                }
                            };
                            srv.group.push(group);
                        };

                        cfg.server.push(srv);
                    }
                }
                str if str == String::from("instance") => {
                    match serde_json::from_value::<Vec<Instance>>(json!(value)) {
                        Ok(get_instance) => {
                            cfg.instance = get_instance
                        }
                        Err(err) => {
                            return Err(KPGError::new_with_string(KPGConfigParseFailed, format!("instance invalid json format. error: {}", err)));
                        }
                    }
                }
                _ => {
                    warn!("unrecognized configuration type. key: {}", key);
                }
            }
        }

        Ok(cfg)
    }
}