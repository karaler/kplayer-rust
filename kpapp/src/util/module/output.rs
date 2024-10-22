use serde::{Deserialize, Serialize};
use crate::util::module::validator::protocol::*;
use validator::Validate;
use crate::util::module::resource::KPAppResourceItem;

#[derive(Debug, Validate, Clone, Serialize, Deserialize)]
pub struct KPAppOutput {
    pub name: String,
    #[validate(custom(function = "rtmp_or_file_url"))]
    pub path: String,
}