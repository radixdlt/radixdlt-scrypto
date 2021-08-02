extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;

use crate::kernel::*;

/// Represents the severity of a log message.
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// A logger for dumping messages.
pub struct Logger {}

impl Logger {
    pub fn log(level: LogLevel, message: String) {
        let s = match level {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        };
        let input = EmitLogInput {
            level: s.to_string(),
            message,
        };
        let _: EmitLogOutput = call_kernel(EMIT_LOG, input);
    }

    pub fn trace(message: String) {
        Self::log(LogLevel::Trace, message);
    }

    pub fn debug(message: String) {
        Self::log(LogLevel::Debug, message);
    }

    pub fn info(message: String) {
        Self::log(LogLevel::Info, message);
    }

    pub fn warn(message: String) {
        Self::log(LogLevel::Warn, message);
    }

    pub fn error(message: String) {
        Self::log(LogLevel::Error, message);
    }
}
