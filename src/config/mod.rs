use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::RecvError;
use enum_display::EnumDisplay;
use libkplayer::codec::playlist::KPPlayList;
use serde::{Deserialize, Serialize};
use uuid::Version;
use crate::config::version::ParseConfig;
use crate::config::version::version_300::Version300;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGConfigFileOpenFailed;
use crate::util::file::find_existed_file;

pub mod version;
pub mod env;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub version: String,
    pub playlist: Vec<Playlist>,
    pub scene: Vec<Scene>,
    pub output: Vec<Output>,
    pub server: Vec<Server>,
    pub instance: Vec<Instance>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub name: String,
    pub resource: Vec<ResourceType>,
    #[serde(rename = "start_point")]
    pub start_point: usize,
    #[serde(rename = "play_mode")]
    pub play_mode: String,
    #[serde(rename = "universal_seek")]
    pub universal_seek: i64,
    #[serde(rename = "universal_end")]
    pub universal_end: i64,
    #[serde(rename = "universal_reverse_seek")]
    pub universal_reverse_seek: i64,
    #[serde(rename = "universal_reverse_end")]
    pub universal_reverse_end: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    ResourceSingle {
        path: String,
    },
    ResourceDirectory {
        directory: String,
        extension: Vec<String>,
    },
    ResourceDetail {
        path: String,
        name: Option<String>,
        seek: Option<i32>,
        end: Option<i32>,
        stream: Option<Stream>,
    },
}

impl Default for ResourceType {
    fn default() -> Self {
        ResourceType::ResourceSingle {
            path: String::new(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub video: i64,
    pub audio: i64,
    pub subtitle: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub name: String,
    pub group: Vec<Group>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub name: String,
    pub app: String,
    pub params: Params,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    pub text: String,
    pub y: i64,
    #[serde(rename = "font_size")]
    pub font_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub name: String,
    pub source: String,
    pub group: Vec<OutputType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputType {
    OutputSingle { path: String },
    OutputDetail {
        name: String,
        #[serde(rename = "reconnect_internal")]
        reconnect_internal: u32,
        path: String,
    },
}

impl Default for OutputType {
    fn default() -> Self {
        OutputType::OutputSingle {
            path: String::default(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub name: String,
    pub target: ServerType,
    pub group: Vec<ServerGroup>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerGroup {
    pub name: String,
    pub port: u32,
    pub address: String,
    pub token: ServerTokenType,
    pub schema: ServerSchema,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumDisplay, Hash, Ord, PartialOrd, Eq)]
pub enum ServerType {
    None,
    Media,
    Api,
}

impl Default for ServerType {
    fn default() -> Self {
        ServerType::None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumDisplay)]
pub enum ServerTokenType {
    SingleToken {
        token: String
    },
    CSToken {
        server: String,
        client: String,
    },
}

impl Default for ServerTokenType {
    fn default() -> Self {
        ServerTokenType::SingleToken { token: String::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumDisplay, Ord, PartialOrd, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ServerSchema {
    None,
    Rtmp,
    Http,
}

impl Default for ServerSchema {
    fn default() -> Self {
        ServerSchema::None
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub name: String,
    pub playlist: String,
    pub scene: String,
    pub server: String,
    pub cache: Cache,
    pub encode: Encode,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cache {
    pub on: bool,
    #[serde(rename = "cache_directory")]
    pub cache_directory: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Encode {
    pub video: Video,
    pub audio: Audio,
    pub mode: String,
    #[serde(rename = "max_bit_rate")]
    pub max_bit_rate: i64,
    #[serde(rename = "avg_quality")]
    pub avg_quality: u32,
    pub profile: String,
    pub preset: String,
    #[serde(rename = "gop_uint")]
    pub gop_uint: u8,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub width: u16,
    pub height: u16,
    pub fps: u8,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio {
    #[serde(rename = "channel_layout")]
    pub channel_layout: u8,
    pub channels: u8,
    #[serde(rename = "sample_rate")]
    pub sample_rate: u16,
}

pub fn parse_file() -> Result<Root, KPGError> {
    let path = {
        let search_config_path = vec!["config.json", "config.json5"];
        match find_existed_file(search_config_path.clone()) {
            None => {
                return Err(KPGError::new_with_string(KPGConfigFileOpenFailed, format!("not found config file. search_config_path: {:?}", search_config_path)));
            }
            Some(val) => val,
        }
    };
    let mut file = File::open(path.clone()).map_err(|err| {
        KPGError::new_with_string(KPGConfigFileOpenFailed, format!("open config file failed. path: {:?}, error: {:?}", path, err))
    })?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|err| {
        KPGError::new_with_string(KPGConfigFileOpenFailed, format!("read config file failed. path: {:?}, error: {:?}", path, err))
    })?;


    let get_version = "3.0.0";
    let version = {
        let version_300: Box<dyn ParseConfig> = Box::new(Version300::default());
        version_300
    };

    // use json5
    {
        let file_context = String::from_utf8(buf).unwrap();
        let serde_json_parsed: serde_json::Value = json5::from_str(file_context.as_str()).map_err(|err| {
            KPGError::new_with_string(KPGConfigFileOpenFailed, format!("parse json5 format failed. path: {:?}, error: {:?}", path, err))
        })?;
        buf = serde_json_parsed.to_string().into_bytes();
    }


    let cfg = version.parse(&buf)?;
    Ok(cfg)
}