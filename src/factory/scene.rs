use std::collections::HashMap;
use std::path::Path;
use libkplayer::plugin::plugin::KPPlugin;
use log::info;
use serde::{Deserialize, ser, Serialize};
use crate::config::Root;
use crate::factory::KPGFactory;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::{*};
use crate::util::file::{compare_md5, download_file, find_existed_file};
use crate::util::jsonrpc::jsonrpc_call;

impl KPGFactory {
    pub(super) fn create_scene(&mut self, cfg: &Root) -> Result<(), KPGError> {
        self.scene = {
            // create scene
            let mut scenes = HashMap::new();
            for s in cfg.scene.iter() {
                let mut group = HashMap::new();
                for item in s.group.iter() {
                    // check plugin
                    let file_path = KPGFactory::get_plugin_file_path(&item.app);
                    let plugin_information = plugin_center_get_plugin_information(&item.ticket, &item.app)?;

                    if !compare_md5(&file_path, &plugin_information.hash) {
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

// get font result
#[derive(Serialize)]
struct GetFontRequest {
    name: String,
}

#[derive(Deserialize)]
struct GetFontResponse {
    url: String,
}

fn plugin_center_get_font(url: &String, name: &String) -> Result<GetFontResponse, KPGError> {
    let request = GetFontRequest {
        name: name.to_string(),
    };

    let result = jsonrpc_call(url.to_string(), "Resource.GetFont", vec![request])?;
    match serde_json::from_str::<GetFontResponse>(result.as_str()) {
        Ok(get_font) => Ok(get_font),
        Err(err) => {
            Err(KPGError::new_with_string(KPGPluginCenterGetFontFailed, format!("get font failed. name: {}, url: {}, response: {}, error: {}", name, url, result, err)))
        }
    }
}


// get plugin information
#[derive(Serialize)]
struct PluginInformationRequest {
    app: String,
    version: Vec<String>,
}

#[derive(Deserialize)]
struct PluginInformationResponse {
    url: String,
    hash: String,
}

fn plugin_center_get_plugin_information(url: &String, app_name: &String) -> Result<PluginInformationResponse, KPGError> {
    let mut request = PluginInformationRequest {
        app: app_name.to_string(),
        version: vec![],
    };
    for version in KPPlugin::get_support_version() {
        request.version.push(version)
    }

    let result = jsonrpc_call(url.to_string(), "Plugin.GetInformation", vec![request])?;
    match serde_json::from_str::<PluginInformationResponse>(result.as_str()) {
        Ok(plugin_information) => Ok(plugin_information),
        Err(err) => {
            Err(KPGError::new_with_string(KPGPluginCenterGetPluginInformationFailed, format!("get plugin information parse json format failed. app: {}, url: {}, response: {}, error: {}", app_name, url, result, err)))
        }
    }
}