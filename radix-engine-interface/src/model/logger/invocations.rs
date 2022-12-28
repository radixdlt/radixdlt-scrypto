use sbor::rust::fmt;
use sbor::rust::fmt::Debug;

use crate::api::api::*;
use crate::model::{LoggerInvocation, NativeInvocation, SerializedInvocation};
use crate::scrypto;
use crate::wasm::*;
use sbor::rust::string::String;
use sbor::*;

/// Represents the level of a log message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeId, Encode, Decode, crate::Describe)]
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

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
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

impl Into<SerializedInvocation> for LoggerLogInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Logger(LoggerInvocation::Log(self)).into()
    }
}
