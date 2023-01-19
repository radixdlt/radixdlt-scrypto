use radix_engine_interface::model::{CallTableInvocation, LoggerInvocation, NativeInvocation};
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;

use crate::api::wasm::*;
use crate::api::*;
use crate::*;
use sbor::rust::string::String;
use sbor::*;

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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LoggerLogInvocation {
    pub level: Level,
    pub message: String,
}

impl Invocation for LoggerLogInvocation {
    type Output = ();
}

impl SerializableInvocation for LoggerLogInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for LoggerLogInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Logger(LoggerInvocation::Log(self)).into()
    }
}
