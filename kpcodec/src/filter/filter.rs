use std::collections::BTreeMap;
use log::error;
use serde::{Deserialize, Serialize};
use crate::filter::*;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct KPFilter {
    name: String,
    filter_name: String,
    arguments: BTreeMap<String, String>,
    allow_arguments: Vec<String>,
    #[serde(skip)]
    filter: KPAVFilter,
}

impl std::fmt::Debug for KPFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KPFilter")
            .field("filter_name", &self.filter_name)
            .field("arguments", &self.arguments)
            .field("allow_arguments", &self.allow_arguments)
            .finish_non_exhaustive()
    }
}

impl KPFilter {
    pub fn new<T: ToString>(name: T, filter_name_t: T, arguments: BTreeMap<String, String>, allow_arguments: Vec<String>) -> Result<Self> {
        let name = name.to_string();
        let filter_name = filter_name_t.to_string();
        let filter = unsafe { avfilter_get_by_name(cstring!(filter_name).as_ptr()) };
        if filter.is_null() {
            return Err(anyhow!("find filter by name failed. name: {}", filter_name));
        }
        Ok(KPFilter {
            name,
            filter_name,
            arguments,
            allow_arguments,
            filter: KPAVFilter::from(filter),
        })
    }

    pub fn create_by_graph(&self, filter_graph: &KPAVFilterGraph) -> Result<KPAVFilterContext> {
        assert!(!self.filter.is_null());
        let mut filter_context: *mut AVFilterContext = ptr::null_mut();
        let ret = unsafe {
            avfilter_graph_create_filter(
                &mut filter_context,
                self.filter.as_ptr(),
                cstring!(self.name.clone()).as_ptr(),
                cstring!(self.format_arguments()).as_ptr(),
                ptr::null_mut(),
                filter_graph.get())
        };
        if ret < 0 {
            return Err(anyhow!("create filter by graph failed. error: {:?}", averror!(ret)));
        }
        assert!(!filter_context.is_null());

        Ok(KPAVFilterContext::from(filter_context))
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_filter_name(&self) -> &String {
        &self.filter_name
    }

    pub fn format_arguments(&self) -> String {
        let mut arg = String::default();
        for (index, (first, second)) in self.arguments.iter().enumerate() {
            if index != 0 {
                arg += ":";
            }
            if first.is_empty() {
                arg += &second.clone().replace(":", r"\:");
            } else if second.is_empty() {
                arg += &first.clone().replace(":", r"\:");
            } else {
                let str = format!("{}={}", &first.clone().replace(":", r"\:"), &second.clone().replace(":", r"\:")).to_string();
                arg += str.as_str();
            }
        }
        arg
    }
}