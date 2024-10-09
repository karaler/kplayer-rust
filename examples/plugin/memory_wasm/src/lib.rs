use std::alloc::{alloc, dealloc, Layout};
use std::slice;

#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    unsafe {
        let layout = Layout::from_size_align(size, 1).unwrap();
        let ptr = alloc(layout);
        ptr
    }
}

#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, size: usize) {
    unsafe {
        let layout = Layout::from_size_align(size, 1).unwrap();
        dealloc(ptr, layout);
    }
}

#[no_mangle]
pub extern "C" fn print_string(ptr: *const u8, len: usize) {
    let slice = unsafe { slice::from_raw_parts(ptr, len) };
    if let Ok(string) = std::str::from_utf8(slice) {
        println!("Received string: {}", string);
    }
}