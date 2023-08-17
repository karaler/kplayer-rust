use std::collections::HashMap;
use libkplayer::plugin::plugin::KPPlugin;
use libkplayer::server::media_pusher::KPMediaPusher;
use log::info;
use crate::config::{OutputType, Root};
use crate::factory::KPGFactory;
use crate::util::error::KPGError;
use crate::util::rand::rand_string;

impl KPGFactory {
    pub(super) fn create_output(&mut self, cfg: &Root) -> Result<(), KPGError> {
        self.output = {
            let mut outputs = HashMap::new();
            for out in cfg.output.iter() {
                let mut media_pusher = KPMediaPusher::new(out.name.clone());
                for group in &out.group {
                    match group {
                        OutputType::OutputSingle { path } => {
                            media_pusher.add_pusher(rand_string(8), path.clone(), 0);
                        }
                        OutputType::OutputDetail {
                            name,
                            reconnect_internal,
                            path,
                        } => {
                            media_pusher.add_pusher(name, path.clone(), reconnect_internal.clone());
                        }
                    }
                }
                info!("create output success. name: {}, source: {}, output: {:?}",out.name,out.source,out.group);
                outputs.insert(out.name.clone(), media_pusher);
            }
            outputs
        };
        Ok(())
    }
}