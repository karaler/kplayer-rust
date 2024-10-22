use log::info;
use crate::init::initialize;
use crate::util::*;
use crate::util::context::KPAppContext;
use crate::util::module::resource::{KPAppResourceItem, ResourceItem, SingleDetail};
use crate::util::module::resource::ResourceItem::Single;
use crate::util::module::scene::KPAppPlugin;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KPAppConfig {
    pub playlist: KPAppResource,
    pub output: KPAppOutput,
    pub scene: KPAppScene,
}

impl KPAppConfig {
    pub fn from_json_str(json_str: String) -> Result<KPAppConfig> {
        Ok(serde_json::from_str(json_str.as_str())?)
    }
}

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