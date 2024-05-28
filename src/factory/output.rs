use crate::config::{OutputType, Root};
use crate::factory::KPGFactory;
use crate::server::ServerContext;
use crate::util::error::KPGError;
use crate::util::rand::rand_string;
use anyhow::Result;
use libkplayer::bindings::{getiopolicy_np, nan};
use libkplayer::codec::transform::KPTransform;
use libkplayer::plugin::plugin::KPPlugin;
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(super) struct KPGOutput {
    name: String,
    source_path: String,
    push_path: String,
}

impl KPGOutput {
    pub fn new<T: ToString>(name: T, source_path: T, push_path: T) -> Self {
        KPGOutput {
            name: name.to_string(),
            source_path: source_path.to_string(),
            push_path: push_path.to_string(),
        }
    }

    pub async fn serve(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_push_path(&self) -> String {
        self.push_path.clone()
    }

    pub fn get_source_path(&self) -> String {
        self.source_path.clone()
    }
}

impl KPGFactory {
    pub(super) async fn create_output(&mut self, cfg: &Root) -> Result<(), KPGError> {
        Ok(())
    }
}
