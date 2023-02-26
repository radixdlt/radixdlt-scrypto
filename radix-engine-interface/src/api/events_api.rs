use crate::abi::LegacyDescribe;
use crate::data::ScryptoEncode;

use super::types::Hash;

pub trait ClientEventsApi<E> {
    fn emit_event<T: ScryptoEncode + LegacyDescribe>(&mut self, event: T) -> Result<(), E>;

    fn emit_raw_event(&mut self, schema_hash: Hash, event_data: Vec<u8>) -> Result<(), E>;
}
