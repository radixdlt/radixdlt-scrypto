use crate::types::*;

/// A non-fungible is a piece of data that is uniquely identified within a resource.
#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct NonFungible {
    immutable_data: Vec<u8>,
    mutable_data: Vec<u8>,
}

impl NonFungible {
    pub fn new(immutable_data: Vec<u8>, mutable_data: Vec<u8>) -> Self {
        Self {
            immutable_data,
            mutable_data,
        }
    }

    pub fn immutable_data(&self) -> Vec<u8> {
        self.immutable_data.clone()
    }

    pub fn mutable_data(&self) -> Vec<u8> {
        self.mutable_data.clone()
    }

    pub fn set_mutable_data(&mut self, new_mutable_data: Vec<u8>) {
        self.mutable_data = new_mutable_data;
    }
}
