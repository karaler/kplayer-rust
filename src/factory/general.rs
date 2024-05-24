use crate::config::env::get_homedir;
use crate::factory::{KPGFactory, ThreadResult, PLUGIN_DIRECTORY, PLUGIN_EXTENSION};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGFactoryOpenPluginFailed;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

impl KPGFactory {
    pub fn get_plugin_file_path(plugin_name: &String) -> PathBuf {
        let mut file_path = get_homedir();
        file_path.push(PathBuf::from(PLUGIN_DIRECTORY));
        file_path.push(format!("{}{}", plugin_name, PLUGIN_EXTENSION));

        file_path
    }

    pub fn get_font_file_path<T: ToString>(font_name: T) -> PathBuf {
        let mut file_path = get_homedir();
        file_path.push(PathBuf::from(PLUGIN_DIRECTORY));
        file_path.push(format!("{}.ttf", font_name.to_string()));

        file_path
    }

    pub fn read_plugin_content(plugin_name: &String) -> Result<Vec<u8>, KPGError> {
        let mut data = Vec::new();

        let file_path = KPGFactory::get_plugin_file_path(plugin_name);
        let mut fs = File::open(Path::new(file_path.to_str().unwrap())).map_err(|err| {
            KPGError::new_with_string(
                KPGFactoryOpenPluginFailed,
                format!(
                    "open plugin file failed. path: {}, error: {}",
                    file_path.to_str().unwrap(),
                    err
                ),
            )
        })?;
        fs.read_to_end(&mut data).map_err(|err| {
            KPGError::new_with_string(
                KPGFactoryOpenPluginFailed,
                format!(
                    "read plugin file failed. path: {}, error: {}",
                    file_path.to_str().unwrap(),
                    err
                ),
            )
        })?;

        Ok(data)
    }

    pub fn get_instance_source(name: String, port: u16) -> String {
        format!("rtmp://127.0.0.1:{}/live/{}", port, name)
    }

    pub fn get_instance_cache_path(name: &String) -> String {
        format!("cache/{}", name)
    }
}
