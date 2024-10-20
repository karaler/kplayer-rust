use crate::scene::*;
use crate::scene::engine::wasm::KPEngine;

#[derive(Default)]
pub struct KPScene {
    pub media_type: KPAVMediaType,
    groups: Vec<Vec<KPFilter>>,
    pub sort_type: KPSceneSortType,
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
    pub fn new(media_type: KPAVMediaType, groups: Vec<Vec<KPFilter>>, sort_type: KPSceneSortType) -> Self {
        KPScene { media_type, groups, sort_type }
    }

    pub fn from_engine(engine: &KPEngine) -> Self {
        KPScene {
            media_type: engine.media_type.clone(),
            groups: engine.groups.clone(),
            sort_type: engine.sort_type.clone(),
        }
    }

    pub fn add_group(&mut self, group: Vec<KPFilter>) {
        self.groups.push(group);
    }

    pub fn get_filters(&self) -> Vec<Vec<KPFilter>> {
        self.groups.clone()
    }
}