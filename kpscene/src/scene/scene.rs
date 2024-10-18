use crate::scene::*;
use crate::scene::engine::wasm::KPEngine;

#[derive(Default)]
pub struct KPScene {
    pub media_type: KPAVMediaType,
    groups: Vec<Vec<KPFilter>>,
}

impl KPScene {
    pub fn new(media_type: KPAVMediaType, groups: Vec<Vec<KPFilter>>) -> Self {
        KPScene { media_type, groups }
    }

    pub fn from_engine(engine: &KPEngine) -> Self {
        KPScene {
            media_type: engine.media_type.clone(),
            groups: engine.groups.clone(),
        }
    }

    pub fn add_group(&mut self, group: Vec<KPFilter>) {
        self.groups.push(group);
    }

    pub fn get_filters(&self) -> Vec<Vec<KPFilter>> {
        self.groups.clone()
    }
}