use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use libkplayer::bindings::getiopolicy_np;
use libkplayer::codec::transform::KPTransform;
use libkplayer::plugin::plugin::KPPlugin;
use log::info;
use crate::config::{OutputType, Root};
use crate::factory::KPGFactory;
use crate::util::error::KPGError;
use crate::util::rand::rand_string;

#[derive(Clone)]
pub(super) struct KPGOutput {
    source_name: String,
    push_path: String,
}

impl KPGOutput {
    pub fn new(source_name: String, push_path: String) -> Self {
        KPGOutput {
            source_name,
            push_path,
        }
    }

    pub fn get_media_pusher(&self) -> String {
        self.push_path.clone()
    }

    pub fn get_source_name(&self) -> String {
        self.source_name.clone()
    }
}

impl KPGFactory {
    pub(super) fn create_output(&mut self, cfg: &Root) -> Result<(), KPGError> {
        Ok(())
    }
}