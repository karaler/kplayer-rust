use std::env;
use log::{info, LevelFilter};
use crate::init::initialize;
use crate::util::*;
use crate::util::common::generate_unique_string;

#[derive(Clone, Debug)]
pub struct KPAppContext {
    pub home_dir: PathBuf,
    pub config_path: PathBuf,
    pub plugin_sub_path: PathBuf,
    pub plugin_extension: String,
    pub config: KPAppConfig,

    // state
    pub temporarily_server_app: String,
}

impl KPAppContext {
    pub fn new(home_dir: PathBuf, config_path: PathBuf) -> Result<Self> {
        if !config_path.exists() {
            return Err(anyhow!("The kplayer.json file does not exist at the provided home path. file_path: {}", config_path.display()));
        }
        let kplayer_json_content = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("Failed to read kplayer.json file: {}", e))?;
        let config = KPAppConfig::from_json_str(kplayer_json_content)?;

        // context
        Ok(KPAppContext {
            plugin_sub_path: home_dir.join("plugin"),
            plugin_extension: ".kpe".to_string(),
            home_dir,
            config_path,
            config,
            temporarily_server_app: generate_unique_string(),
        })
    }
}


#[test]
fn load_context() -> Result<()> {
    initialize();

    let homedir = PathBuf::from(env::var("HOME_PATH")?);
    let config_path = homedir.join(PathBuf::from("kplayer.json"));
    let context = KPAppContext::new(homedir, config_path)?;
    info!("{:?}",context);
    Ok(())
}