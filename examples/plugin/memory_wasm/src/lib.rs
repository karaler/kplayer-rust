use std::slice;
use kplayer_plugin_sdk::memory::{allocate, MemoryPoint};
use kplayer_plugin_sdk::{memory_combine, memory_split};

#[no_mangle]
pub extern "C" fn print_string(memory_point: MemoryPoint) {
    let (ptr, size) = memory_split!(memory_point);
    let slice = unsafe { slice::from_raw_parts(ptr, size) };
    if let Ok(string) = std::str::from_utf8(slice) {
        println!("Received string: {}", string);
    }
}

#[no_mangle]
pub extern "C" fn get_string() -> MemoryPoint {
    let str = "Hello KPlayer!";
    let memory_p = allocate(str.len());
    let (ptr, size) = memory_split!(memory_p);

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
    assert_eq!(a, a1 as i32);
    assert_eq!(b, b1);
}