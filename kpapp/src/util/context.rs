use std::env;
use log::info;
use crate::init::initialize;
use crate::util::*;

#[derive(Clone, Debug)]
pub struct KPAppContext {
    pub home_path: PathBuf,
    pub plugin_sub_path: PathBuf,
    pub plugin_extension: String,
    pub config: KPAppConfig,
}

impl KPAppContext {
    pub fn new<T: ToString>(home_path: Option<T>) -> Result<Self> {
        let home_path = match home_path {
            None => {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
            }
            Some(p) => {
                let path = PathBuf::from(p.to_string());
                if !path.exists() && path.is_dir() {
                    return Err(anyhow!("The provided path does not exist. path: {}", p.to_string()));
                }
                path
            }
        };

        // get config
        let kplayer_json_path = home_path.join("kplayer.json");
        if !kplayer_json_path.exists() {
            return Err(anyhow!("The kplayer.json file does not exist at the provided home path."));
        }
        let kplayer_json_content = std::fs::read_to_string(&kplayer_json_path)
            .map_err(|e| anyhow!("Failed to read kplayer.json file: {}", e))?;
        let config = KPAppConfig::from_json_str(kplayer_json_content)?;

        // context
        Ok(KPAppContext {
            plugin_sub_path: home_path.join("plugin"),
            plugin_extension: ".kpe".to_string(),
            home_path,
            config,
        })
    }
}


#[test]
fn load_context() -> Result<()> {
    initialize();

    let home_path = env::var("HOME_PATH")?;
    let context = KPAppContext::new(Some(home_path))?;
    info!("{:?}",context);
    Ok(())
}