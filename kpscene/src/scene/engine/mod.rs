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

mod wasm;
mod version;
mod vars;
mod caller;
mod inject;