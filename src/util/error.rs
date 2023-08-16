use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use enum_display::EnumDisplay;

#[derive(Debug, Eq, PartialEq, EnumDisplay, Copy, Clone, Hash)]
pub enum KPGErrorCode {
    KPGErrorCodeNone = 0,
    KPGConfigParseOpenFileFailed = -1900000,
    KPGConfigParseFailed,
    KPGConfigFileOpenFailed,
    KPGInstanceLaunchFailed,
    KPGPlayListAddMediaFailed,
    KPGMediaServerExited,
    KPGFactoryParseConfigFailed,
    KPGFactoryOpenPluginFailed,
    KPGServerMediaServerEnableSchemaFailed,
    KPGServerMediaServerStartFailed,
    KPGServerMediaServerStopFailed,
    KPGUtilReadDirectoryFailed,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KPGError {
    code: KPGErrorCode,
    error: String,
}

impl Error for KPGError {}

impl fmt::Display for KPGError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "code: {}, error: {}", self.code.to_string(), self.error)
    }
}

impl KPGError {
    pub fn new(code: KPGErrorCode, error: &dyn Error) -> KPGError {
        let err_content = error.to_string();
        return KPGError {
            code,
            error: err_content,
        };
    }

    pub fn new_with_string(code: KPGErrorCode, error: String) -> KPGError {
        return KPGError {
            code,
            error: error,
        };
    }

    pub fn new_with_str(code: KPGErrorCode, error: &'static str) -> KPGError {
        return KPGError {
            code,
            error: error.to_string(),
        };
    }

    pub fn equal(&self, code: KPGErrorCode) -> bool {
        self.code == code
    }
}
