use crate::abi::*;
use crate::*;

pub struct Logger {}

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;

impl Logger {
    pub fn log(level: String, message: String) {
        let input = EmitLogInput { level, message };
        let _: EmitLogOutput = call_kernel!(EMIT_LOG, input);
    }

    pub fn trace(message: String) {
        Self::log("TRACE".to_string(), message);
    }

    pub fn debug(message: String) {
        Self::log("DEBUG".to_string(), message);
    }

    pub fn info(message: String) {
        Self::log("INFO".to_string(), message);
    }

    pub fn warn(message: String) {
        Self::log("WARN".to_string(), message);
    }

    pub fn error(message: String) {
        Self::log("ERROR".to_string(), message);
    }
}
