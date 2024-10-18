use rusty_ffmpeg::ffi::*;
use crate::util::alias::*;
use log::info;
use std::fmt::{Display, Formatter};
use rusty_ffmpeg::ffi::*;
use crate::util::alias::*;
use anyhow::{anyhow, Result};
use strum_macros::{Display, EnumString};
use std::collections::{BTreeMap, HashMap};
use std::ffi::c_int;
use std::ptr;
use crate::{averror, cstr, cstring};

pub mod alias;
pub mod codec_status;
pub mod encode_parameter;