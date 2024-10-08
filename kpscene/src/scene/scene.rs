use crate::scene::*;

#[derive(Default)]
pub struct KPScene {
    groups: Vec<Vec<KPFilter>>,
}

impl KPScene {
    pub fn new(groups: Vec<Vec<KPFilter>>) -> Self {
        KPScene { groups }
    }

    pub fn add_group(&mut self, group: Vec<KPFilter>) {
        self.groups.push(group);
    }

    pub fn get_filters(&self) -> Vec<Vec<KPFilter>> {
        self.groups.clone()
    }
}