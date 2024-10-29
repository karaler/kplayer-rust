use anyhow::Result;
use crate::util::module::output::KPAppOutput;
use crate::util::module::resource::KPAppResource;
use crate::util::module::scene::KPAppScene;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::anyhow;
use crate::util::config::KPAppConfig;

pub mod context;
pub mod config;
pub mod module;
pub mod vars;
mod common;