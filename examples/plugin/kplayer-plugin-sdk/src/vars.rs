use std::collections::{BTreeMap, HashMap};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub enum KPPluginMediaType {
    AVMEDIA_TYPE_VIDEO,
    AVMEDIA_TYPE_AUDIO,
}

#[derive(Default, Clone, Debug, PartialOrd, PartialEq)]
pub enum KPSceneSortType {
    #[default]
    After,
    Before,
}

impl KPSceneSortType {
    pub fn to_i32(&self) -> i32 {
        match self {
            KPSceneSortType::After => 0,
            KPSceneSortType::Before => 1,
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct KPPluginInfo {
    pub name: String,
    pub filter_name: String,
    pub default_arguments: BTreeMap<String, String>,
    pub allow_arguments: Vec<String>,
}