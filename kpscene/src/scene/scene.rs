use crate::scene::*;
use crate::scene::engine::wasm::KPEngine;

pub struct KPScene {
    pub media_type: KPAVMediaType,
    groups: Vec<Vec<KPFilter>>,
    pub sort_type: KPSceneSortType,
    engine: KPEngine,
}

#[derive(Default, Clone, Debug, PartialOrd, PartialEq)]
pub enum KPSceneSortType {
    #[default]
    After,
    Before,
}

impl KPSceneSortType {
    pub fn from_i32(t: i32) -> KPSceneSortType {
        match t {
            0 => KPSceneSortType::After,
            1 => KPSceneSortType::Before,
            _ => KPSceneSortType::After, // Default case
        }
    }
}

impl KPScene {
    pub fn new(engine: KPEngine) -> Self {
        KPScene {
            media_type: engine.media_type.clone(),
            groups: engine.groups.clone(),
            sort_type: engine.sort_type.clone(),
            engine,
        }
    }

    pub fn add_group(&mut self, group: Vec<KPFilter>) {
        self.groups.push(group);
    }

    pub fn get_filters(&self) -> Vec<Vec<KPFilter>> {
        self.groups.clone()
    }

    pub async fn get_update_argument(&self, arguments: BTreeMap<String, String>) -> Result<BTreeMap<String, BTreeMap<String, String>>> {
        self.engine.get_update_command(arguments).await
    }
}