use serde::Serialize;

pub mod decode;
mod util;
mod init;
mod filter;
mod encode;

#[macro_export]
macro_rules! cstring {
    ($path:expr) => {{
        std::ffi::CString::new($path.clone()).unwrap()
    }};
}

#[macro_export]
macro_rules! cstr {
    ($ptr:expr) => {{
        unsafe {
            std::ffi::CStr::from_ptr($ptr).to_str().unwrap().to_string()
        }
    }};
}

#[derive(Clone, Debug)]
pub struct KPAVError {
    pub code: u64,
    pub error: String,
    pub message: String,
}

#[macro_export]
macro_rules! averror {
    ($ret:expr) => {
        {
            let mut errbuf = [0u8; 4096];
            let message = {
                unsafe { rusty_ffmpeg::ffi::av_make_error_string(errbuf.as_mut_ptr() as *mut i8, errbuf.len(), $ret); }
                let c_str = unsafe { std::ffi::CStr::from_ptr(errbuf.as_ptr() as *const std::ffi::c_char) };
                c_str.to_string_lossy().into_owned()
            };
            let error = {
               unsafe { rusty_ffmpeg::ffi::av_strerror($ret, errbuf.as_mut_ptr() as *mut i8, errbuf.len()); }
                let c_str = unsafe { std::ffi::CStr::from_ptr(errbuf.as_ptr() as *const std::ffi::c_char) };
                c_str.to_string_lossy().into_owned()
            };
            crate::KPAVError { code: $ret as u64 , message, error }
        }
    };
}

#[macro_export]
macro_rules! mut_ptr {
    ($ptr:expr) => {
        if $ptr.is_null() {
            std::ptr::null_mut()
        } else {
            $ptr
        }
    };
}