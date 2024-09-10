use crate::util::*;

pub struct KPService {
    config: Vec<KPConfig>,
}

impl Default for KPService {
    fn default() -> Self {
        KPService {
            config: vec![KPConfig::rtmp {
                address: IpAddr::from_str("0.0.0.0").unwrap(),
                port: 1935,
                gop_number: 1,
            }]
        }
    }
}

impl KPService {
    pub fn new() -> Self {
        KPService {
            config: Vec::new(),
        }
    }

    pub fn append(&mut self, cfg: KPConfig) {
        self.config.push(cfg)
    }

    pub fn get_config(&self) -> &Vec<KPConfig> {
        &self.config
    }
}
