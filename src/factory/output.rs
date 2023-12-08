use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use libkplayer::bindings::getiopolicy_np;
use libkplayer::codec::transform::KPTransform;
use libkplayer::plugin::plugin::KPPlugin;
use libkplayer::server::media_pusher::KPMediaPusher;
use log::info;
use crate::config::{OutputType, Root};
use crate::factory::KPGFactory;
use crate::util::error::KPGError;
use crate::util::rand::rand_string;

#[derive(Clone)]
pub(super) struct KPGOutput {
    source_name: String,
    media_pusher: Arc<Mutex<KPMediaPusher>>,
}

impl KPGOutput {
    pub fn new(source_name: String, media_pusher: KPMediaPusher) -> Self {
        KPGOutput {
            source_name,
            media_pusher: Arc::new(Mutex::new(media_pusher)),
        }
    }

    pub fn get_media_pusher(&self) -> Arc<Mutex<KPMediaPusher>> {
        self.media_pusher.clone()
    }

    pub fn get_source_name(&self) -> String {
        self.source_name.clone()
    }
}

impl KPGFactory {
    pub(super) fn create_output(&mut self, cfg: &Root) -> Result<(), KPGError> {
        self.output = {
            let mut outputs = HashMap::new();
            for out in cfg.output.iter() {
                let mut media_pusher = KPMediaPusher::new(out.name.clone());
                let source = out.source.clone();

                // determine if the source is an instance name
                let source_url = match self.instance.get(&source) {
                    None => {
                        source
                    }
                    Some(get_ins) => {
                        let transform = { get_ins.lock().unwrap().transform.clone() };
                        let ins = transform.lock().unwrap();
                        ins.get_output_url()
                    }
                };

                // set source url
                media_pusher.set_source(source_url.clone());

                // add output
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
                outputs.insert(out.name.clone(), KPGOutput::new(source_url.clone(), media_pusher));
                info!("create output success. name: {}, source: {}, output: {:?}", out.name, source_url, out.group);
            }
            outputs
        };
        Ok(())
    }
}