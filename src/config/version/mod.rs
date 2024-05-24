use crate::config::Root;
use crate::util::error::KPGError;

pub mod version_300;

pub trait ParseConfig {
    fn parse(&self, cfg: &Vec<u8>) -> Result<Root, KPGError>;
}
