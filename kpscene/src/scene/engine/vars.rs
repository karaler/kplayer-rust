use crate::scene::engine::*;

#[derive(Default, Debug, PartialOrd, PartialEq)]
pub enum KPEngineStatus {
    #[default]
    None,
    Initialized,
    Loaded,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct KPPluginInfo {
    pub name: String,
    pub filter_name: String,
    pub default_arguments: BTreeMap<String, String>,
    pub allow_arguments: Vec<String>,
}