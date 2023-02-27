use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::logger::{Level, LoggerLogInput, LOGGER_LOG_IDENT};
use radix_engine_interface::data::scrypto_encode;
use sbor::rust::string::String;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// A utility for logging messages.
#[derive(Debug)]
pub struct Logger {}

impl Logger {
    /// Emits a log to console.
    pub fn log(level: Level, message: String) {
        ScryptoEnv
            .call_method(
                RENodeId::Logger,
                LOGGER_LOG_IDENT,
                scrypto_encode(&LoggerLogInput { level, message }).unwrap(),
            )
            .unwrap();
    }

    /// Emits a trace message.
    pub fn trace(message: String) {
        Self::log(Level::Trace, message);
    }

    /// Emits a debug message.
    pub fn debug(message: String) {
        Self::log(Level::Debug, message);
    }

    /// Emits an info message.
    pub fn info(message: String) {
        Self::log(Level::Info, message);
    }

    /// Emits a warn message.
    pub fn warn(message: String) {
        Self::log(Level::Warn, message);
    }

    /// Emits an error message.
    pub fn error(message: String) {
        Self::log(Level::Error, message);
    }
}
