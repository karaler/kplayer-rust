use std::slice;
use crate::*;

pub type MemoryPoint = u64;

pub const INVALID_MEMORY_POINT: MemoryPoint = 0;

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

pub(crate) fn read_memory(memory_point: &MemoryPoint) -> Vec<u8> {
    let (ptr, size) = memory_split!(memory_point);
    let mut buffer = vec![0; size];

    unsafe {
        let slice = slice::from_raw_parts_mut(ptr, size);
        buffer.copy_from_slice(slice);
    }
    buffer
}

pub(crate) fn read_memory_as_string(memory_point: MemoryPoint) -> Result<String, String> {
    let buf = read_memory(&memory_point);
    let result = if let Ok(str) = std::str::from_utf8(&buf) {
        str.to_string()
    } else {
        return Err("data is not valid UTF-8".to_string());
    };

    // destroy memory
    deallocate(memory_point);
    Ok(result)
}