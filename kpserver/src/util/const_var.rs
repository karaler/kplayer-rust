use strum_macros::{Display, EnumString};

#[derive(Debug, Display, EnumString)]
pub enum KPProtocol {
    #[strum(serialize = "rtmp")]
    Rtmp
}