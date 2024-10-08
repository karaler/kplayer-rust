use strum_macros::Display;

#[derive(Clone, Display, Debug)]
pub enum KPServerMessage {
    rtmp_start {
        name: String,
    },
    rtmp_stop {
        name: String,
        error: Option<String>,
    },
    httpflv_start {
        name: String,
    },
    httpflv_stop {
        name: String,
        error: Option<String>,
    },
    hls_start {
        name: String,
    },
    hls_stop {
        name: String,
        error: Option<String>,
    },
    rtmp_pull_start {
        name: String,
        source_url: String,
        error: Option<String>,
    },
    rtmp_pull_stop {
        name: String,
        source: String,
        error: Option<String>,
    },
}