use crate::crypto::Hash;
use crate::sbor::rust::vec::Vec;

pub trait ClientEventApi<E> {
    fn emit_event(&mut self, schema_hash: Hash, event_data: Vec<u8>) -> Result<(), E>;
}
