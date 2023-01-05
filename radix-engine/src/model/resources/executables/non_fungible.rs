use crate::types::*;
use radix_engine_interface::data::ScryptoValue;
use radix_engine_interface::scrypto;

/// A non-fungible is a piece of data that is uniquely identified within a resource.
#[scrypto(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct NonFungible {
    immutable_data: ScryptoValue,
    mutable_data: ScryptoValue,
}

impl NonFungible {
    pub fn new(immutable_data: ScryptoValue, mutable_data: ScryptoValue) -> Self {
        Self {
            immutable_data,
            mutable_data,
        }
    }

    pub fn immutable_data(&self) -> ScryptoValue {
        self.immutable_data.clone()
    }

    pub fn mutable_data(&self) -> ScryptoValue {
        self.mutable_data.clone()
    }

    pub fn set_mutable_data(&mut self, new_mutable_data: ScryptoValue) {
        self.mutable_data = new_mutable_data;
    }
}
