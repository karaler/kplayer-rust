use std::collections::HashMap;
use kpcodec::util::alias::KPAVMediaType;
use crate::scene::engine::version::KPEngineVersion;
use tokio::sync::{Mutex};
use wasmtime::{Instance, Linker, Store};
use kpcodec::filter::filter::KPFilter;
use crate::scene::engine::vars::KPEngineStatus;
use anyhow::Result;
use anyhow::anyhow;
use std::ops::{Deref, DerefMut};
use crate::scene::engine::wasm::KPEngine;
use std::{env, fs};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use log::info;
use wasmtime_wasi::preview1::WasiP1Ctx;
use crate::init::initialize;
use crate::memory_split;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

pub mod wasm;
pub(crate) mod version;
pub(crate) mod vars;
pub(crate) mod caller;
pub(crate) mod inject;

pub(crate) type MemoryPoint = u64;

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
            (ptr as *mut u8, size as usize)
        }
    };
}