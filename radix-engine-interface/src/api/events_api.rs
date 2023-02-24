use crate::abi::LegacyDescribe;
use crate::data::ScryptoEncode;
use sbor::rust::fmt::Debug;

pub trait ClientEventsApi<E: Debug> {
    fn emit_event<T: LegacyDescribe + ScryptoEncode>(&mut self, event: T) -> Result<(), E>;
}
