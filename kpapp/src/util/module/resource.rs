use std::collections::HashMap;
use log::info;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;
use serde_json::Value;
use crate::util::module::validator::file::*;
use validator::Validate;
use kpcodec::util::alias::KPAVMediaType;
use crate::util::common::generate_unique_string;
use super::validator::resource::validate_unique_names;

#[derive(Debug, Validate, Clone, Serialize, Deserialize)]
pub struct SingleDetail {
    #[validate(custom(function = "exist_file"))]
    #[validate(custom(function = "video_extension"))]
    pub path: String,
    pub expect_streams: HashMap<KPAVMediaType, Option<usize>>,
}

#[derive(Serialize, Clone, Debug)]
pub enum ResourceItem {
    Single {
        single: SingleDetail
    },
}


impl<'de> Deserialize<'de> for ResourceItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;
        match value {
            Value::String(path) => {
                Ok(ResourceItem::Single {
                    single: SingleDetail {
                        path,
                        expect_streams: Default::default(),
                    },
                })
            }
            Value::Object(map) => {
                let single_value = map.get("Single").ok_or_else(|| D::Error::custom("Missing 'Single' in ResourceItem object"))?;
                let single_map = match single_value {
                    Value::Object(map) => map,
                    _ => return Err(D::Error::custom("Invalid type within 'Single' object")),
                };
                let single_map_detail = single_map.get("single").ok_or_else(|| D::Error::custom("Missing 'single' object within 'Single'"))?;
                let single: SingleDetail = serde_json::from_value(single_map_detail.clone()).map_err(D::Error::custom)?;
                Ok(ResourceItem::Single { single })
            }
            _ => Err(D::Error::custom("Invalid type for ResourceItem")),
        }
    }
}


#[derive(Serialize, Clone, Debug)]
pub struct KPAppResourceItem {
    pub name: String,
    pub resource: ResourceItem,
}

impl<'de> Deserialize<'de> for KPAppResourceItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;
        match value {
            Value::String(path) => {
                Ok(KPAppResourceItem {
                    name: generate_unique_string(),
                    resource: ResourceItem::Single {
                        single: SingleDetail {
                            path,
                            expect_streams: Default::default(),
                        },
                    },
                })
            }
            Value::Object(map) => {
                let name = map.get("name").ok_or_else(|| D::Error::custom("Missing field 'name'"))?.as_str().ok_or_else(|| D::Error::custom("Field 'name' is not a string"))?.to_string();;

                // extract and deserialize the resource field
                let resource_value = map.get("resource").ok_or_else(|| D::Error::custom("Missing field 'resource'"))?;
                let resource: ResourceItem = serde_json::from_value(resource_value.clone()).map_err(D::Error::custom)?;

                // Construct and return the KPAppResourceItem
                Ok(KPAppResourceItem {
                    name,
                    resource,
                })
            }
            _ => Err(D::Error::custom("Invalid type for ResourceItem")),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate)]
pub struct KPAppResource {
    pub name: String,
    #[validate(custom(function = "validate_unique_names"))]
    pub list: Vec<KPAppResourceItem>,
}