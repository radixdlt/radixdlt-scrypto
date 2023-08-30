use radix_engine_interface::types::Level;
use sbor::rust::string::String;

use crate::engine::scrypto_env::ScryptoVmV1Api;

/// A utility for logging messages.
#[derive(Debug)]
pub struct Logger {}

impl Logger {
    /// Emits a log to console.
    pub fn emit_log(level: Level, message: String) {
        ScryptoVmV1Api::sys_log(level, message);
    }

    /// Emits a trace message.
    pub fn trace(message: String) {
        Self::emit_log(Level::Trace, message);
    }

    /// Emits a debug message.
    pub fn debug(message: String) {
        Self::emit_log(Level::Debug, message);
    }

    /// Emits an info message.
    pub fn info(message: String) {
        Self::emit_log(Level::Info, message);
    }

    /// Emits a warn message.
    pub fn warn(message: String) {
        Self::emit_log(Level::Warn, message);
    }

    /// Emits an error message.
    pub fn error(message: String) {
        Self::emit_log(Level::Error, message);
    }
}
