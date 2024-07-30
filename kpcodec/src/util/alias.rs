use std::collections::HashMap;
use std::ptr;
use rusty_ffmpeg::ffi::{*};
use crate::cstring;

// KPFormatContextPtr
pub struct KPFormatContextPtr(pub *mut AVFormatContext);

impl Default for KPFormatContextPtr {
    fn default() -> Self {
        KPFormatContextPtr(ptr::null_mut())
    }
}

impl Drop for KPFormatContextPtr {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { avformat_free_context(self.0); }
        }
    }
}

impl KPFormatContextPtr {
    pub fn new() -> Self {
        KPFormatContextPtr(unsafe { avformat_alloc_context() })
    }
    pub fn get(&self) -> &mut AVFormatContext {
        unsafe { self.0.as_mut().unwrap() }
    }
}

// KPCodecContextPtr
pub struct KPCodecContextPtr(pub *mut AVCodecContext);

impl Default for KPCodecContextPtr {
    fn default() -> Self {
        KPCodecContextPtr(ptr::null_mut())
    }
}

// KPAVDictionary
pub struct KPAVDictionary(pub *mut *mut AVDictionary);

impl Drop for KPAVDictionary {
    fn drop(&mut self) {
        unsafe { av_dict_free(self.0); }
    }
}

impl KPAVDictionary {
    pub fn new<T: ToString>(values: &HashMap<T, T>) -> Self {
        let mut dict: *mut AVDictionary = ptr::null_mut();
        unsafe {
            for (key, value) in values {
                av_dict_set(&mut dict, cstring!(key.to_string()).as_ptr(), cstring!(value.to_string()).as_ptr(), 0);
            }
        }

        KPAVDictionary(&mut dict)
    }
    pub fn get(&self) -> *mut *mut AVDictionary {
        self.0
    }
}