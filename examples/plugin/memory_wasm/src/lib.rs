use std::alloc::{alloc, dealloc, Layout};
use std::slice;

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
            (ptr, size)
        }
    };
}

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

#[no_mangle]
pub extern "C" fn get_string() -> u64 {
    let str = "Hello KPlayer!";
    let size = str.len();
    let ptr = allocate(size);

    unsafe {
        let slice = slice::from_raw_parts_mut(ptr, size);
        slice.copy_from_slice(str.as_bytes());
    };
    memory_combine!(ptr, size)
}

#[test]
fn test_memory_combine() {
    let a = 100;
    let b = 200;
    let c = memory_combine!(a,b);
    let (a1, b1) = memory_split!(c);
    assert_eq!(a, a1);
    assert_eq!(b, b1);
}