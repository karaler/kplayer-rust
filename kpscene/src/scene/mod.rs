use kpcodec::filter::filter::KPFilter;
use kpcodec::filter::graph::KPGraph;
use crate::scene::scene::KPScene;
use anyhow::Result;
use anyhow::anyhow;
use std::collections::{BTreeMap, HashMap};
use std::env;
use log::info;
use rusty_ffmpeg::ffi::AV_PIX_FMT_YUV420P;
use kpcodec::decode::decode::KPDecode;
use kpcodec::encode::encode::KPEncode;
use kpcodec::filter::graph::KPGraphStatus;
use kpcodec::util::alias::{KPAVMediaType, KPAVPixelFormat, KPAVSampleFormat};
use kpcodec::util::encode_parameter::KPEncodeParameter;
use crate::init::initialize;

mod scene;
mod graph;
mod engine;