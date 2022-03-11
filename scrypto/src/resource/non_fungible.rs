use crate::resource::*;
use crate::resource_def;
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
    /// Returns the resource definition.
    pub fn resource_def_id(&self) -> ResourceDefId {
        self.address.resource_def_id()
    }

    /// Returns the non-fungible ID.
    pub fn key(&self) -> NonFungibleId {
        self.address.key()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        resource_def!(self.resource_def_id()).get_non_fungible_data(&self.key())
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T, auth: Proof) {
        resource_def!(self.resource_def_id()).update_non_fungible_data(&self.key(), new_data, auth);
    }
}
