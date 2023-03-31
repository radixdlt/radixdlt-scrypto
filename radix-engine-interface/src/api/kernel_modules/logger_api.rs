use crate::api::types::Level;
use crate::sbor::rust::string::String;

pub trait ClientLoggerApi<E> {
    fn log_message(&mut self, level: Level, message: String) -> Result<(), E>;
}
