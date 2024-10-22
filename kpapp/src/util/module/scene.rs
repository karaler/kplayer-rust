use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::util::module::validator::protocol::*;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KPAppPlugin {
    pub name: String,
    pub arguments: BTreeMap<String, String>,
}

#[derive(Debug, Validate, Clone, Serialize, Deserialize)]
pub struct KPAppScene {
    pub name: String,
    pub list: Vec<KPAppPlugin>,
}