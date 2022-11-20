use crate::model::{convert, ComponentStateSubstate, MethodAuthorization};
use crate::types::*;
use radix_engine_interface::abi::Type;
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::AccessRules;

/// A transient resource container.
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct MetadataSubstate {
    pub metadata: HashMap<String, String>,
}

impl MetadataSubstate {
    pub fn insert(
        &mut self,
        key: String,
        value: String,
    ) {
        self.metadata.insert(key, value);
    }
}
