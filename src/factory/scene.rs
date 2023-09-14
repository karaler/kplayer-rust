use std::collections::HashMap;
use libkplayer::plugin::plugin::KPPlugin;
use libkplayer::util::kpcodec::kpfilter_parameter::KPFilterParameterItem;
use log::info;
use crate::config::Root;
use crate::factory::KPGFactory;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGFactoryOpenPluginFailed;

impl KPGFactory {
    pub(super) fn create_scene(&mut self, cfg: &Root) -> Result<(), KPGError> {
        self.scene = {
            let mut scenes = HashMap::new();
            for s in cfg.scene.iter() {
                let mut group = HashMap::new();
                for item in s.group.iter() {
                    let mut plugin = KPPlugin::new(KPGFactory::read_plugin_content(&item.app)?);
                    plugin.set_custom_params(item.params.clone());
                    group.insert(item.name.clone(), plugin);
                }
                info!("create scene success. scene: {}", s.name.clone());
                scenes.insert(s.name.clone(), group);
            }
            scenes
        };
        Ok(())
    }
}