use std::net::IpAddr;

#[derive(Clone)]
pub enum KPConfig {
    rtmp {
        address: IpAddr,
        port: usize,
        gop_number: usize,
    },
    httpflv {
        port: usize,
    },
    hls {
        port: usize,
    },
}
