use crate::api::types::*;
use crate::data::IndexedScryptoValue;
use sbor::rust::vec::Vec;

pub trait ClientComponentApi<E> {
    fn invoke_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<IndexedScryptoValue, E>;
}
