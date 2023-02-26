use crate::abi::LegacyDescribe;
use crate::data::ScryptoEncode;

pub trait ClientEventsApi<E> {
    fn emit_event<T: ScryptoEncode + LegacyDescribe>(&mut self, event: T) -> Result<(), E>;
}
