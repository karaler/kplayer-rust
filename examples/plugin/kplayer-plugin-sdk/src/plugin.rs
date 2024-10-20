use std::collections::HashMap;
use crate::plugin_item::KPPluginItem;
use crate::vars::{KPPluginMediaType, KPSceneSortType};

pub static mut INSTANCE_PTR: *mut KPPlugin = 0x0 as *mut KPPlugin;

pub struct KPPlugin {
    pub app: String,
    pub author: String,
    pub version: String,
    pub media_type: KPPluginMediaType,
    pub sort_type: KPSceneSortType,
    pub items: Vec<Vec<Box<dyn KPPluginItem>>>,
}

impl KPPlugin {
    pub fn init<T: ToString>(app: T, author: T, version: T, media_type: KPPluginMediaType, sort_type: KPSceneSortType, items: Vec<Vec<Box<dyn KPPluginItem>>>) {
        unsafe {
            assert_eq!(INSTANCE_PTR, 0x0 as *mut KPPlugin);
            let plugin = Box::new(KPPlugin {
                app: app.to_string(),
                author: author.to_string(),
                version: version.to_string(),
                media_type,
                sort_type,
                items,
            });
            let ptr: &'static mut KPPlugin = Box::leak(plugin);
            INSTANCE_PTR = ptr as *mut KPPlugin
        }
    }

    pub fn get() -> &'static mut KPPlugin {
        unsafe {
            assert_ne!(INSTANCE_PTR, 0x0 as *mut KPPlugin);
            &mut *INSTANCE_PTR
        }
    }

    pub fn push_group(&mut self, group: Vec<Box<dyn KPPluginItem>>) {
        self.items.push(group)
    }
}