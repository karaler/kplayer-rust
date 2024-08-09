use crate::filter::*;

#[derive(Default, Clone)]
pub struct KPFilter {
    filter_name: String,
    arguments: HashMap<String, String>,
    filter: KPAVFilter,
}

impl KPFilter {
    pub fn new<T: ToString>(filter_name_t: T, arguments: HashMap<String, String>) -> Result<Self> {
        let filter_name = filter_name_t.to_string();
        let filter = unsafe { avfilter_get_by_name(cstring!(filter_name).as_ptr()) };
        if filter.is_null() {
            return Err(anyhow!("find filter by name failed. name: {}", filter_name));
        }
        Ok(KPFilter {
            filter_name,
            arguments,
            ..Default::default()
        })
    }

    pub fn create_by_graph(&self, filter_graph: &KPAVFilterGraph) -> Result<KPAVFilterContext> {
        let filter_context = KPAVFilterContext::default();
        let ret = unsafe {
            avfilter_graph_create_filter(
                &mut filter_context.as_ptr(),
                self.filter.get(),
                cstring!(self.filter_name.clone()).as_ptr(),
                cstring!(self.format_arguments()).as_ptr(),
                ptr::null_mut(),
                filter_graph.get())
        };
        if ret < 0 {
            return Err(anyhow!("create filter by graph failed. error: {:?}", averror!(ret)));
        }
        Ok(filter_context)
    }

    fn format_arguments(&self) -> String {
        let mut arg = String::default();
        for (index, (first, second)) in self.arguments.iter().enumerate() {
            if index != 0 {
                arg += ":";
            }
            if first.is_empty() {
                arg += &first.clone().replace(":", r"\:");
            } else {
                let str = format!("{}={}", &first.clone().replace(":", r"\:"), &second.clone().replace(":", r"\:")).to_string();
                arg += str.as_str();
            }
        }
        arg
    }
}