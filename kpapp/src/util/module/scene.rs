use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::util::module::validator::protocol::*;
use validator::Validate;
use crate::util::module::deserialize::string::map_string_or_number;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct KPAppPlugin {
    pub name: String,
    #[serde(deserialize_with = "map_string_or_number")]
    pub arguments: BTreeMap<String, String>,
}

#[derive(Debug, Validate, Clone, Serialize, Deserialize)]
pub struct KPAppScene {
    pub name: String,
    #[validate(nested)]
    pub list: Vec<KPAppPlugin>,
}