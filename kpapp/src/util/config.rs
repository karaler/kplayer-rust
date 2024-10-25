use validator::Validate;
use crate::util::*;

#[derive(Serialize, Deserialize, Clone, Debug, Validate)]
pub struct KPAppConfig {
    #[validate(nested)]
    pub playlist: KPAppResource,
    #[validate(nested)]
    pub output: KPAppOutput,
    #[validate(nested)]
    pub scene: KPAppScene,
}

impl KPAppConfig {
    pub fn from_json_str(json_str: String) -> Result<KPAppConfig> {
        let cfg: KPAppConfig = serde_json::from_str(json_str.as_str())?;
        cfg.validate()?;
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;
    use crate::init::initialize;
    use crate::util::config::KPAppConfig;
    use crate::util::module::resource::{KPAppResource, KPAppResourceItem, SingleDetail};
    use crate::util::module::resource::ResourceItem::Single;
    use anyhow::{anyhow, Result};
    use log::info;
    use crate::util::context::KPAppContext;
    use crate::util::module::output::KPAppOutput;
    use crate::util::module::scene::{KPAppPlugin, KPAppScene};

    #[test]
    fn output_json() -> Result<()> {
        initialize();

        let context = KPAppConfig {
            playlist: KPAppResource { name: "default_playlist".to_string(), list: vec![KPAppResourceItem { name: "default_media".to_string(), resource: Single { single: SingleDetail { path: "media_path".to_string(), expect_streams: Default::default() } } }] },
            output: KPAppOutput { name: "default_output".to_string(), path: "rtmp://127.0.0.1:1935/live/test".to_string() },
            scene: KPAppScene { name: "default_scene".to_string(), list: vec![KPAppPlugin { name: "text".to_string(), arguments: Default::default() }] },
        };

        let json_str = serde_json::to_string(&context)?;
        info!("{}", json_str);
        Ok(())
    }

    #[test]
    fn parse_json_file() -> Result<()> {
        initialize();
        let home_path = PathBuf::from(env::var("HOME_PATH")?);
        let kplayer_json_path = home_path.join("kplayer.json");
        if !kplayer_json_path.exists() {
            return Err(anyhow!("The kplayer.json file does not exist at the provided home path."));
        }
        let kplayer_json_content = std::fs::read_to_string(&kplayer_json_path)
            .map_err(|e| anyhow!("Failed to read kplayer.json file: {}", e))?;
        let config = KPAppConfig::from_json_str(kplayer_json_content)?;

        let output_str = serde_json::to_string_pretty(&config)?;
        println!("{}", output_str);

        Ok(())
    }
}
