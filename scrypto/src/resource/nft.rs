use sbor::*;

use crate::resource::*;
use crate::rust::marker::PhantomData;
use crate::types::*;

/// Represents an NFT unit.
#[derive(Debug)]
pub struct Nft<I: Encode + Decode, M: Encode + Decode> {
    resource_address: Address,
    id: u128,
    immutable_data: PhantomData<I>,
    mutable_data: PhantomData<M>,
}

impl<I: Encode + Decode, M: Encode + Decode> Nft<I, M> {
    pub fn new(resource_address: Address, id: u128) -> Self {
        Self {
            resource_address,
            id,
            immutable_data: PhantomData,
            mutable_data: PhantomData,
        }
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> Address {
        self.resource_address
    }

    /// Returns the NFT ID.
    pub fn id(&self) -> u128 {
        self.id
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> (I, M) {
        ResourceDef::from(self.resource_address()).get_nft_data(self.id)
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: (I, M), auth: BucketRef) {
        ResourceDef::from(self.resource_address()).update_nft_data(self.id, new_data, auth);
    }
}
