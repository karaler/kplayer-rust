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
use crate::{averror};
use std::env;
use log::info;
use std::collections::{BTreeMap, LinkedList};
use std::ffi::c_int;
use log::{debug, warn};
use std::collections::VecDeque;
use log::trace;
use std::ffi::c_char;
use std::ffi::c_uint;
use crate::filter::filter::KPFilter;
use crate::filter::graph::KPGraphSourceAttribute;

pub mod filter;
pub mod graph;

pub trait KPGraphSource {
    fn get_source(&self, media_type: &KPAVMediaType) -> Result<KPGraphSourceAttribute>;
}