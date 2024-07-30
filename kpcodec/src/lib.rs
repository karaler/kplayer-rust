pub mod decode;
mod util;
mod init;

#[macro_export]
macro_rules! cstring {
    ($path:expr) => {
        std::ffi::CString::new($path.clone()).unwrap_or_else(|err| {
            panic!("Failed to convert path to CString: {}", err)
        })
    };
}

#[macro_export]
macro_rules! cstr {
    ($ptr:expr) => {{
        unsafe {
            std::ffi::CStr::from_ptr($ptr).to_str().unwrap()
        }
    }};
}