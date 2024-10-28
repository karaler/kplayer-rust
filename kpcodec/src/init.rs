use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::Once;
use dotenv::dotenv;
use libc::size_t;
use log::LevelFilter;
use rusty_ffmpeg::ffi::{av_log_set_level, AV_LOG_QUIET, AV_LOG_PANIC, AV_LOG_FATAL, AV_LOG_ERROR, AV_LOG_WARNING, AV_LOG_INFO, AV_LOG_VERBOSE, AV_LOG_DEBUG, AV_LOG_TRACE, av_log_set_callback, va_list, vsnprintf, av_log_get_level};
use crate::{cstr, cstring};

static INIT: Once = Once::new();

pub(crate) fn initialize() {
    INIT.call_once(|| {
        dotenv().ok();
    });

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
}

// This will be the callback function for FFmpeg logs
unsafe extern "C" fn ffmpeg_log_callback(ptr: *mut ::std::os::raw::c_void, level: ::std::os::raw::c_int, fmt: *const ::std::os::raw::c_char, args: va_list) {
    if level > av_log_get_level() {
        return;
    }

    let log_level = ffmpeg_level_to_log_level(level);
    let buffer_size: std::os::raw::c_ulong = 1024;
    let mut buffer = vec![0 as c_char; buffer_size as usize];
    let len = vsnprintf(buffer.as_mut_ptr(), buffer_size, fmt, args);
    if len > 0 {
        let message = CStr::from_ptr(buffer.as_ptr()).to_string_lossy();
        let trimmed_message = message.trim_end();
        if !(trimmed_message.starts_with("forced frame type") && trimmed_message.contains("was changed to frame type")) {
            log::log!(log_level,"[Core] {}", trimmed_message);
        }
    }
}

// This function will map FFmpeg log levels to Rust log levels
fn ffmpeg_level_to_log_level(level: c_int) -> log::Level {
    match level as u32 {
        AV_LOG_PANIC | AV_LOG_FATAL => log::Level::Error,
        AV_LOG_ERROR => log::Level::Error,
        AV_LOG_WARNING => log::Level::Warn,
        AV_LOG_INFO => log::Level::Info,
        AV_LOG_VERBOSE => log::Level::Debug,
        AV_LOG_DEBUG => log::Level::Debug,
        AV_LOG_TRACE => log::Level::Trace,
        _ => log::Level::Error,
    }
}

fn log_level_to_ffmpeg_level(log_level: log::Level) -> c_int {
    match log_level {
        log::Level::Error => AV_LOG_ERROR as c_int,
        log::Level::Warn => AV_LOG_WARNING as c_int,
        log::Level::Info => AV_LOG_INFO as c_int,
        log::Level::Debug => AV_LOG_DEBUG as c_int,
        log::Level::Trace => AV_LOG_TRACE as c_int,
    }
}

pub fn set_ff_logger(level_filter: Option<log::LevelFilter>) {
    unsafe {
        let current_log_level = level_filter.unwrap_or(log::max_level());
        av_log_set_level(log_level_to_ffmpeg_level(current_log_level.to_level().unwrap()));
        av_log_set_callback(Some(ffmpeg_log_callback));
    }
}
