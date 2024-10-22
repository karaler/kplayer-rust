use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::util::module::validator::file::*;
use validator::Validate;
use kpcodec::util::alias::KPAVMediaType;

#[derive(Debug, Validate, Clone, Serialize, Deserialize)]
pub struct SingleDetail {
    #[validate(custom(function = "exist_file"))]
    #[validate(custom(function = "video_extension"))]
    pub path: String,
    pub expect_streams: HashMap<KPAVMediaType, Option<usize>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ResourceItem {
    Single {
        single: SingleDetail
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KPAppResourceItem {
    pub name: String,
    pub resource: ResourceItem,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KPAppResource {
    pub name: String,
    pub list: Vec<KPAppResourceItem>,
}