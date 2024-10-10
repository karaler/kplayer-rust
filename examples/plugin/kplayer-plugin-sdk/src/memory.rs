use std::slice;
use crate::*;

pub type MemoryPoint = u64;

#[no_mangle]
pub extern "C" fn allocate(size: usize) -> MemoryPoint {
    unsafe {
        let layout = Layout::from_size_align(size, 1).unwrap();
        let ptr = alloc(layout);
        memory_combine!(ptr, size)
    }
}

pub fn allocate_string<T: ToString>(s: T) -> MemoryPoint {
    let str = s.to_string();
    let memory_p = allocate(str.len());
    let (ptr, size) = memory_split!(memory_p);

    unsafe {
        let slice = slice::from_raw_parts_mut(ptr, size);
        slice.copy_from_slice(str.as_bytes());
    };
    memory_p
}

#[no_mangle]
pub extern "C" fn deallocate(memory_p: MemoryPoint) {
    let (ptr, size) = memory_split!(memory_p);
    unsafe {
        let layout = Layout::from_size_align(size, 1).unwrap();
        dealloc(ptr, layout);
    }
}