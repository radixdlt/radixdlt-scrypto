use radix_engine_interface::{api::ClientLoggerApi, types::Level};
use sbor::rust::string::String;

use crate::engine::scrypto_env::ScryptoEnv;

/// A utility for logging messages.
#[derive(Debug)]
pub struct Logger {}

impl Logger {
    /// Emits a log to console.
    pub fn log_message(level: Level, message: String) {
        ScryptoEnv.log_message(level, message).unwrap();
    }

    /// Emits a trace message.
    pub fn trace(message: String) {
        Self::log_message(Level::Trace, message);
    }

    /// Emits a debug message.
    pub fn debug(message: String) {
        Self::log_message(Level::Debug, message);
    }

    /// Emits an info message.
    pub fn info(message: String) {
        Self::log_message(Level::Info, message);
    }

    /// Emits a warn message.
    pub fn warn(message: String) {
        Self::log_message(Level::Warn, message);
    }

    /// Emits an error message.
    pub fn error(message: String) {
        Self::log_message(Level::Error, message);
    }
}
