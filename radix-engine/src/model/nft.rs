use sbor::*;
use scrypto::rust::vec::Vec;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum NftError {
    UnauthorizedAccess,
}

/// A nft is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Nft {
    data: Vec<u8>,
}

impl Nft {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn set_data(&mut self, new_data: Vec<u8>) -> Result<(), NftError> {
        self.data = new_data;
        Ok(())
    }
}
