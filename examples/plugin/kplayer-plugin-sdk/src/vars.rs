use std::collections::{BTreeMap, HashMap};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub enum KPPluginMediaType {
    AVMEDIA_TYPE_VIDEO,
    AVMEDIA_TYPE_AUDIO,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct KPPluginInfo {
    pub filter_name: String,
    pub default_arguments: BTreeMap<String, String>,
    pub allow_arguments: Vec<String>,
}