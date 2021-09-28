use sbor::*;

use crate::kernel::*;
use crate::rust::string::String;

/// A utility for logging messages.
#[derive(Debug)]
pub struct Logger {}

/// Represents a log severity.
#[derive(Debug, Clone, TypeId, Describe, Encode, Decode)]
pub enum Level {
    Error = 0,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Logger {
    pub fn log(level: Level, message: String) {
        let input = EmitLogInput {
            level: level as u8,
            message,
        };
        let _: EmitLogOutput = call_kernel(EMIT_LOG, input);
    }

    pub fn trace(message: String) {
        Self::log(Level::Trace, message);
    }

    pub fn debug(message: String) {
        Self::log(Level::Debug, message);
    }

    pub fn info(message: String) {
        Self::log(Level::Info, message);
    }

    pub fn warn(message: String) {
        Self::log(Level::Warn, message);
    }

    pub fn error(message: String) {
        Self::log(Level::Error, message);
    }
}
