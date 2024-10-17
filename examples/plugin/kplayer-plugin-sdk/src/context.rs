use std::collections::HashMap;
use log::info;
use serde::{Deserialize, Serialize};
use crate::memory::{allocate, allocate_string, MemoryPoint};
use crate::plugin::KPPlugin;
use crate::vars::KPPluginInfo;

#[no_mangle]
pub extern "C" fn get_version() -> MemoryPoint {
    let plugin = KPPlugin::get();
    let memory_point = allocate_string(&plugin.version);
    memory_point
}

#[no_mangle]
pub extern "C" fn get_app() -> MemoryPoint {
    let plugin = KPPlugin::get();
    let memory_point = allocate_string(&plugin.app);
    memory_point
}

#[no_mangle]
pub extern "C" fn get_author() -> MemoryPoint {
    let plugin = KPPlugin::get();
    let memory_point = allocate_string(&plugin.author);
    memory_point
}

#[no_mangle]
pub extern "C" fn get_media_type() -> i32 {
    let plugin = KPPlugin::get();
    plugin.media_type.clone() as i32
}

#[no_mangle]
pub extern "C" fn get_groups() -> MemoryPoint {
    let plugin = KPPlugin::get();
    let mut items: Vec<Vec<KPPluginInfo>> = Vec::new();
    for item in plugin.items.iter() {
        let mut i: Vec<KPPluginInfo> = Vec::new();
        for f_item in item.iter() {
            i.push(KPPluginInfo {
                filter_name: f_item.get_filter_name(),
                default_arguments: f_item.default_arguments(),
                allow_arguments: f_item.allow_arguments(),
            })
        }
        items.push(i);
    }
    let serialized_arguments = serde_json::to_string(&items).unwrap();
    let memory_point = allocate_string(&serialized_arguments);
    memory_point
}