use std::alloc::{alloc, dealloc, Layout};

#[macro_export]
macro_rules! memory_combine {
    ($ptr:expr, $size:expr) => {
        (($ptr as u64) << 32) | ($size as u64)
    };
}
#[macro_export]
macro_rules! memory_split {
    ($value:expr) => {
        {
            let ptr = ($value >> 32) as i32;
            let size = ($value & 0xFFFFFFFF) as u32;
            (ptr as *mut u8, size as usize)
        }
    };
}

pub mod memory;
pub mod plugin;
pub mod vars;
pub mod plugin_item;
mod context;

