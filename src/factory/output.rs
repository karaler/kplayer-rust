use crate::config::{OutputType, Root};
use crate::factory::KPGFactory;
use crate::util::error::KPGError;
use crate::util::rand::rand_string;
use libkplayer::bindings::{getiopolicy_np, nan};
use libkplayer::codec::transform::KPTransform;
use libkplayer::plugin::plugin::KPPlugin;
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use xiu::config::{Config, RtmpConfig, RtmpPullConfig, RtmpPushConfig};
use xiu::service::Service;
use crate::server::ServerContext;
use anyhow::Result;

pub(super) struct KPGOutput {
    name: String,
    config: Config,
    service: Service,
    source_path: String,
    push_path: String,
}

impl KPGOutput {
    pub fn new<T: ToString>(name: T, source_path: T, push_path: T) -> Self {
        let mut cfg = Config::new(0, 0, 0, 0, 0, "error".to_string());
        cfg.rtmp = Some(RtmpConfig {
            enabled: true,
            port: 0,
            gop_num: None,
            pull: Some(RtmpPullConfig {
                enabled: true,
                address: source_path.to_string(),
                port: 1935,
            }),
            push: Some(vec![
                RtmpPushConfig {
                    enabled: true,
                    address: push_path.to_string(),
                    port: 1935,
                }
            ]),
            auth: None,
        });
        let service = Service::new(cfg.clone());

        KPGOutput {
            name: name.to_string(),
            config: cfg,
            service,
            source_path: source_path.to_string(),
            push_path: push_path.to_string(),
        }
    }

    pub async fn serve(&mut self) -> Result<()> {
        self.service.run().await?;
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