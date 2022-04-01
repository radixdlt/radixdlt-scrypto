use crate::resource::*;
use crate::resource_manager;
use crate::rust::marker::PhantomData;

/// Represents a non-fungible unit.
#[derive(Debug)]
pub struct NonFungible<T: NonFungibleData> {
    address: NonFungibleAddress,
    data: PhantomData<T>,
}

impl<T: NonFungibleData> From<NonFungibleAddress> for NonFungible<T> {
    fn from(address: NonFungibleAddress) -> Self {
        Self {
            address,
            data: PhantomData,
        }
    }
}

impl<T: NonFungibleData> NonFungible<T> {
    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        self.address.resource_address()
    }

    /// Returns the non-fungible ID.
    pub fn id(&self) -> NonFungibleId {
        self.address.non_fungible_id()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        resource_manager!(self.resource_address()).get_non_fungible_data(&self.id())
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T) {
        resource_manager!(self.resource_address()).update_non_fungible_data(&self.id(), new_data);
    }
}
