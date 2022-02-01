use crate::resource::*;
use crate::rust::marker::PhantomData;
use crate::types::*;

/// Represents an NFT unit.
#[derive(Debug)]
pub struct Nft<T: NftData> {
    resource_address: Address,
    id: NftKey,
    data: PhantomData<T>,
}

impl<T: NftData> From<(Address, NftKey)> for Nft<T> {
    fn from(tuple: (Address, NftKey)) -> Self {
        Self {
            resource_address: tuple.0,
            id: tuple.1.clone(),
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
    pub fn id(&self) -> NftKey {
        self.id.clone()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        ResourceDef::from(self.resource_address()).get_nft_data(&self.id)
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T, auth: BucketRef) {
        ResourceDef::from(self.resource_address()).update_nft_data(&self.id, new_data, auth);
    }
}
