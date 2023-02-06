use crate::api::types::*;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;

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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Logger(LoggerFn::Log))
    }
}

impl SerializableInvocation for LoggerLogInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Logger(LoggerFn::Log)
    }
}

impl Into<CallTableInvocation> for LoggerLogInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Logger(LoggerInvocation::Log(self)).into()
    }
}
