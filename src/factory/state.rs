use crate::factory::{KPGFactory, KPGFactoryInstance};
use libkplayer::codec::playlist::KPPlayList;
use std::collections::HashMap;

impl KPGFactory {
    pub fn get_server_list(&self) -> Vec<String> {
        let mut server_name = Vec::new();
        for (name, _) in &self.server {
            server_name.push(name.clone());
        }

        server_name
    }

    pub fn get_instance_list(&self) -> Vec<String> {
        let mut instance_name = Vec::new();
        for (name, _) in self.instance.iter() {
            instance_name.push(name.clone());
        }

        instance_name
    }

    pub fn get_instance(&self, name: &String) -> Option<KPGFactoryInstance> {
        match self.instance.get(name) {
            None => None,
            Some(instance) => Some(instance.clone()),
        }
    }

    pub fn get_output_list(&self) -> Vec<String> {
        let mut output_name = Vec::new();
        for (name, _) in self.output.iter() {
            output_name.push(name.clone())
        }

        output_name
    }

    pub fn get_playlist(&self) -> HashMap<String, KPPlayList> {
        self.playlist.clone()
    }
}
