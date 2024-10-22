use crate::scene::*;
use crate::scene::engine::wasm::KPEngine;

pub struct KPScene {
    engines: BTreeMap<String, KPEngine>,
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
    pub fn new() -> Self {
        KPScene {
            engines: Default::default(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&KPEngine> {
        self.engines.values()
    }

    pub fn iter_key(&self) -> impl Iterator<Item=&String> {
        self.engines.keys()
    }

    pub fn add_engine<T: ToString>(&mut self, name: T, engine: KPEngine) {
        self.engines.insert(name.to_string(), engine);
    }

    pub fn get_filters(&self, name: &String) -> Result<Vec<Vec<KPFilter>>> {
        let engine = match self.engines.get(name) {
            None => return Err(anyhow!("name not found")),
            Some(e) => e,
        };
        Ok(engine.groups.clone())
    }

    pub fn get_update_argument<T: ToString>(&self, name: T, arguments: BTreeMap<String, String>) -> Result<BTreeMap<String, BTreeMap<String, String>>> {
        let engine = match self.engines.get(&name.to_string()) {
            None => return Err(anyhow!("name not found")),
            Some(e) => e,
        };
        futures::executor::block_on(async {
            engine.get_update_command(arguments).await
        })
    }
}