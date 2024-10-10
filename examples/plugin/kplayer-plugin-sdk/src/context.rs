use crate::memory::{allocate, allocate_string, MemoryPoint};
use crate::plugin::KPPlugin;

#[no_mangle]
pub extern "C" fn get_app() -> MemoryPoint {
    let plugin = KPPlugin::get();
    let memory_point = allocate_string(&plugin.app);
    memory_point
}