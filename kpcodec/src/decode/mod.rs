use std::collections::HashMap;
use rusty_ffmpeg::ffi::{*};
use std::ffi::{CStr, CString};
use std::ptr;
use std::time::Duration;
use crate::util::alias::{*};
use crate::util::codec_status::KPCodecStatus;
use anyhow::{anyhow, Result};
use crate::{cstr, cstring};
use crate::init::initialize;

mod decode;