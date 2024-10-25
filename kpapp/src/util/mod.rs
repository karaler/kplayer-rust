use anyhow::Result;
use crate::util::module::output::KPAppOutput;
use crate::util::module::resource::KPAppResource;
use crate::util::module::scene::KPAppScene;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::anyhow;
use crate::util::config::KPAppConfig;

pub(crate) mod context;
pub(crate) mod config;
pub(crate) mod module;
pub(crate) mod vars;
mod common;