use std::net::IpAddr;
use std::time::Duration;

#[derive(Clone)]
pub enum KPConfig {
    rtmp {
        name: String,
        address: IpAddr,
        port: usize,
        gop_number: usize,
    },
    httpflv {
        name: String,
        port: usize,
    },
    hls {
        name: String,
        port: usize,
    },
    rtmp_pull {
        name: String,
        source_url: String,
        app_name: Option<String>,
        stream_name: Option<String>,
        keep_alive: bool,
        timeout: Option<Duration>,
        retry_interval: Option<Duration>,
    },
    rtmp_push {
        name: String,
        app_name: String,
        stream_name: String,
        sink_url: String,
        timeout: Option<Duration>,
        retry_interval: Option<Duration>,
    },
}
