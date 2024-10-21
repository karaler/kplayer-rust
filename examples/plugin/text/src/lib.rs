use std::collections::BTreeMap;
use kplayer_plugin_sdk::plugin::KPPlugin;
use kplayer_plugin_sdk::plugin_item::KPPluginItem;
use kplayer_plugin_sdk::vars::{KPPluginMediaType, KPSceneSortType};

#[derive(Default)]
pub struct KPPluginText {}

impl KPPluginItem for KPPluginText {
    fn get_name(&self) -> String {
        "text".to_string()
    }

    fn get_filter_name(&self) -> String {
        "drawtext".to_string()
    }

    fn default_arguments(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        map.insert("text", "hello kplayer");
        map.insert("box", "0");
        map.insert("boxborderw", "10");
        map.insert("boxcolor", "white");
        map.insert("line_spacing", "0");
        map.insert("fontcolor", "white");
        map.insert("fontfile", "resource/font.ttf");
        map.insert("alpha", "1");
        map.insert("fontsize", "17");
        map.insert("shadowcolor", "white");
        map.insert("shadowx", "0");
        map.insert("shadowy", "0");
        map.insert("x", "0");
        map.insert("y", "0");

        map.iter().map(|(k, v)| { (k.to_string(), v.to_string()) }).collect()
    }

    fn allow_arguments(&self) -> Vec<String> {
        let allow = vec!["text", "fontsize", "x", "y"];
        allow.iter().map(|item| { item.to_string() }).collect()
    }

    fn update_commands(&mut self, arguments: &BTreeMap<String, String>) -> BTreeMap<String, String> {
        let mut params = BTreeMap::new();
        let allow_arguments = self.allow_arguments();
        for (key, value) in arguments {
            if allow_arguments.contains(key) {
                params.insert(key.clone(), value.clone());
            }
        }

        let mut cmd = BTreeMap::new();
        cmd.insert("reinit".to_string(), params.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join(";"));
        cmd
    }
}

#[no_mangle]
pub extern "C" fn init() -> i32 {
    match KPPlugin::init("text", "kplayer", "1.0.0", KPPluginMediaType::AVMEDIA_TYPE_VIDEO, KPSceneSortType::After, vec![
        vec![
            Box::new(KPPluginText::default())
        ]
    ], ) {
        Ok(_) => 0,
        Err(err) => -1,
    }
}