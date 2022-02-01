use crate::resource::*;
use crate::rust::marker::PhantomData;
use crate::types::*;

/// Represents an NFT unit.
#[derive(Debug)]
pub struct Nft<T: NftData> {
    resource_address: Address,
    key: NftKey,
    data: PhantomData<T>,
}

impl<T: NftData> From<(Address, NftKey)> for Nft<T> {
    fn from(tuple: (Address, NftKey)) -> Self {
        Self {
            resource_address: tuple.0,
            key: tuple.1.clone(),
            data: PhantomData,
        }
    }
}

impl<T: NftData> Nft<T> {
    /// Returns the resource address.
    pub fn resource_address(&self) -> Address {
        self.resource_address
    }

    /// Returns the NFT ID.
    pub fn key(&self) -> NftKey {
        self.key.clone()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        ResourceDef::from(self.resource_address()).get_nft_data(&self.key)
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T, auth: BucketRef) {
        ResourceDef::from(self.resource_address()).update_nft_data(&self.key, new_data, auth);
    }
}
