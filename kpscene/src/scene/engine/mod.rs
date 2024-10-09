use std::collections::HashMap;
use kpcodec::util::alias::KPAVMediaType;
use crate::scene::engine::version::KPEngineVersion;
use tokio::sync::{Mutex};
use wasmtime::{Instance, Linker, Store};
use wasmtime_wasi::WasiCtx;
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
use tokio::sync::MutexGuard;
use wasmtime_wasi::preview1::WasiP1Ctx;
use crate::init::initialize;
use crate::memory_split;

mod wasm;
mod version;
mod vars;
mod caller;
mod inject;

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
            (ptr, size)
        }
    };
}