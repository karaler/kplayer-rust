use strum_macros::Display;

#[derive(Clone, Display, Debug)]
pub enum KPServerMessage {
    rtmp_start {},
    rtmp_stop {
        error: Option<String>,
    },
    httpflv_start {},
    httpflv_stop {
        error: Option<String>,
    },
    hls_start {},
    hls_stop {
        error: Option<String>,
    },
}