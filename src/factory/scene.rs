use std::collections::HashMap;
use libkplayer::plugin::plugin::KPPlugin;
use log::info;
use reqwest::blocking::Client;
use serde::{Deserialize};
use crate::config::Root;
use crate::factory::KPGFactory;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::{*};
use crate::util::file::{compare_md5, download_file};

impl KPGFactory {
    pub(super) fn create_scene(&mut self, cfg: &Root) -> Result<(), KPGError> {
        self.scene = {
            let mut scenes = HashMap::new();
            for s in cfg.scene.iter() {
                let mut group = HashMap::new();
                for item in s.group.iter() {

                    // check plugin
                    let file_path = KPGFactory::get_plugin_file_path(&item.app);
                    let plugin_information = plugin_center_get_plugin_information(&item.ticket, &item.app)?;

                    if !compare_md5(&file_path, &plugin_information.md5) {
                        if let Err(err) = download_file(&plugin_information.url, &file_path) {
                            return Err(KPGError::new_with_string(KPGPluginCenterDownloadFailed, format!("download plugin file failed. app: {}, url: {}, error: {}", item.app, plugin_information.url, err)));
                        }
                    }

                    // init plugin
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

#[derive(Deserialize)]
struct PluginInformation {
    url: String,
    md5: String,
}

fn plugin_center_get_plugin_information(url: &String, app_name: &String) -> Result<PluginInformation, KPGError> {
    let client = Client::new();
    let response = match client.get(url).send() {
        Ok(res) => res,
        Err(err) => {
            return Err(KPGError::new_with_string(KPGPluginCenterGetPluginInformationFailed, format!("get plugin information failed. app: {}, url: {}, error: {}", app_name, url, err)));
        }
    };

    if !response.status().is_success() {
        return Err(KPGError::new_with_string(KPGPluginCenterGetPluginInformationFailed, format!("get plugin information failed. app: {}, url: {}, status: {}", app_name, url, response.status())));
    }

    let response_data = match response.text() {
        Ok(data) => data,
        Err(err) => {
            return Err(KPGError::new_with_string(KPGPluginCenterGetPluginInformationFailed, format!("get plugin information failed. app: {}, url: {}, error: {}", app_name, url, err)));
        }
    };

    match serde_json::from_str::<PluginInformation>(response_data.as_str()) {
        Ok(plugin_information) => Ok(plugin_information),
        Err(err) => {
            Err(KPGError::new_with_string(KPGPluginCenterGetPluginInformationFailed, format!("get plugin information parse json format failed. app: {}, url: {}, response: {}, error: {}", app_name, url, response_data, err)))
        }
    }
}