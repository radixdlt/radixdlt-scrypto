use radix_engine_common::data::scrypto::ScryptoDescribe;

use crate::crypto::Hash;
use crate::data::scrypto::ScryptoEncode;
use crate::sbor::rust::vec::Vec;

pub trait ClientEventApi<E> {
    fn emit_event<T: ScryptoEncode + ScryptoDescribe>(&mut self, event: T) -> Result<(), E>;

    fn emit_raw_event(&mut self, schema_hash: Hash, event_data: Vec<u8>) -> Result<(), E>;
}
