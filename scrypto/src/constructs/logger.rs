extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;

use crate::kernel::*;

/// Represents the severity of a log message.
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// A logger for dumping messages.
pub struct Logger {}

impl Logger {
    pub fn log(level: Level, message: String) {
        let s = match level {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        };
        let input = EmitLogInput {
            level: s.to_string(),
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
