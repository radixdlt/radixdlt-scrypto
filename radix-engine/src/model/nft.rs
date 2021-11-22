use sbor::*;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::model::Auth;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum NftError {
    UnauthorizedAccess,
}

/// A nft is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Nft {
    data: Vec<u8>,
    update_auth: Address,
}

impl Nft {
    pub fn new(data: Vec<u8>, update_auth: Address) -> Self {
        Self { data, update_auth }
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn set_data(&mut self, new_data: Vec<u8>, auth: Auth) -> Result<(), NftError> {
        if auth.contains(self.update_auth) {
            self.data = new_data;
            Ok(())
        } else {
            Err(NftError::UnauthorizedAccess)
        }
    }
}
