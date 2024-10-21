use std::collections::{BTreeMap, HashMap};
use log::{error, info};
use serde::{Deserialize, Serialize};
use crate::memory::{allocate, allocate_string, read_memory_as_string, MemoryPoint, INVALID_MEMORY_POINT};
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
    plugin.sort_type.to_i32()
}

#[no_mangle]
pub extern "C" fn get_sort_type() -> i32 {
    let plugin = KPPlugin::get();
    plugin.sort_type.to_i32()
}

#[no_mangle]
pub extern "C" fn get_groups() -> MemoryPoint {
    let plugin = KPPlugin::get();
    let mut items: Vec<Vec<KPPluginInfo>> = Vec::new();
    for item in plugin.items.iter() {
        let mut i: Vec<KPPluginInfo> = Vec::new();
        for f_item in item.iter() {
            i.push(KPPluginInfo {
                name: f_item.get_name(),
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

#[no_mangle]
pub extern "C" fn get_update_command(mp: MemoryPoint) -> MemoryPoint {
    let plugin = KPPlugin::get();
    let str = match read_memory_as_string(mp) {
        Ok(str) => str,
        Err(err) => {
            error!("read update command failed. error: {}", err);
            return INVALID_MEMORY_POINT;
        }
    };


    let btree_map_result: Result<BTreeMap<String, String>, serde_json::Error> = serde_json::from_str(&str);
    let argument = match btree_map_result {
        Ok(btree_map) => btree_map,
        Err(err) => {
            error!("Deserialization failed. Error: {}", err);
            return INVALID_MEMORY_POINT;
        }
    };

    // get update command
    let mut update_command: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for item in plugin.items.iter_mut() {
        for plugin_item in item.iter_mut() {
            assert!(update_command.get(&plugin_item.get_name()).is_none());
            let cmd = plugin_item.update_commands(&argument);
            if !cmd.is_empty() {
                update_command.insert(plugin_item.get_name(), cmd);
            }
        }
    }


    let serialized_update_command = match serde_json::to_string(&update_command) {
        Ok(json_str) => json_str,
        Err(err) => {
            error!("Serialization failed. Error: {}", err);
            return INVALID_MEMORY_POINT;
        }
    };
    let memory_point = allocate_string(&serialized_update_command);
    memory_point
}