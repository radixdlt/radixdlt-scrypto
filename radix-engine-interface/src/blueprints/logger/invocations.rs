use crate::api::types::*;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use scrypto_abi::BlueprintAbi;
use sbor::rust::collections::BTreeMap;

pub struct LoggerAbi;

impl LoggerAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}

/// Represents the level of a log message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Categorize, Encode, Decode, LegacyDescribe)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Error => write!(f, "ERROR"),
            Level::Warn => write!(f, "WARN"),
            Level::Info => write!(f, "INFO"),
            Level::Debug => write!(f, "DEBUG"),
            Level::Trace => write!(f, "TRACE"),
        }
    }
}

pub const LOGGER_BLUEPRINT: &str = "Logger";

pub const LOGGER_LOG_IDENT: &str = "log";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LoggerLogInput {
    pub level: Level,
    pub message: String,
}
