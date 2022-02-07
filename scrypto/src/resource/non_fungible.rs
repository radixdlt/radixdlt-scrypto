use crate::resource::*;
use crate::rust::marker::PhantomData;
use crate::types::*;

/// Represents a non-fungible unit.
#[derive(Debug)]
pub struct NonFungible<T: NonFungibleData> {
    resource_address: Address,
    key: NonFungibleKey,
    data: PhantomData<T>,
}

impl<T: NonFungibleData> From<(Address, NonFungibleKey)> for NonFungible<T> {
    fn from(tuple: (Address, NonFungibleKey)) -> Self {
        Self {
            resource_address: tuple.0,
            key: tuple.1.clone(),
            data: PhantomData,
        }
    }
}

impl<T: NonFungibleData> NonFungible<T> {
    /// Returns the resource address.
    pub fn resource_address(&self) -> Address {
        self.resource_address
    }

    /// Returns the non-fungible ID.
    pub fn key(&self) -> NonFungibleKey {
        self.key.clone()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        ResourceDef::from(self.resource_address()).get_non_fungible_data(&self.key)
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T, auth: BucketRef) {
        ResourceDef::from(self.resource_address()).update_non_fungible_data(&self.key, new_data, auth);
    }
}
