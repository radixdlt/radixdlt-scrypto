use crate::sbor::rust::string::String;
use crate::sbor::rust::vec::Vec;

pub trait ClientEventApi<E> {
    fn emit_event(&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), E>;
}
