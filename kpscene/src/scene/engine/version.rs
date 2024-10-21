use std::fmt;
use crate::scene::engine::*;

#[derive(Default)]
pub struct KPEngineVersion {
    version: u32,
}

impl fmt::Debug for KPEngineVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let major = self.version / 10000;
        let minor = (self.version / 100) % 100;
        let patch = self.version % 100;
        write!(f, "{}.{}.{}", major, minor, patch)
    }
}

impl KPEngineVersion {
    // Assuming that the input string follows the format `1.0.00`, we can convert it as follows:
    pub fn from(version: String) -> Result<KPEngineVersion> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() == 3 {
            let major: u32 = parts[0].parse().unwrap();
            let minor: u32 = parts[1].parse().unwrap();
            let patch: u32 = parts[2].parse().unwrap();

            let version = major * 10000 + minor * 100 + patch;
            Ok(KPEngineVersion { version })
        } else {
            Err(anyhow!("Invalid version format. version: {}", version))
        }
    }
}