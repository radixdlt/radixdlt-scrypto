use radix_engine_interface::model::*;
use sbor::rust::marker::PhantomData;

use crate::borrow_resource_manager;
use crate::resource::*;

pub trait ScryptoNonFungibleId {
    /// Creates a non-fungible ID from some uuid.
    fn random() -> Self;
}

impl ScryptoNonFungibleId for NonFungibleId {
    fn random() -> Self {
        let bytes = crate::core::Runtime::generate_uuid().to_be_bytes().to_vec();
        Self::from_bytes(bytes)
    }
}

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

    /// Returns the non-fungible address.
    pub fn address(&self) -> NonFungibleAddress {
        self.address.clone()
    }

    /// Returns the non-fungible ID.
    pub fn id(&self) -> NonFungibleId {
        self.address.non_fungible_id()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        borrow_resource_manager!(self.resource_address()).get_non_fungible_data(&self.id())
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T) {
        borrow_resource_manager!(self.resource_address())
            .update_non_fungible_data(&self.id(), new_data);
    }
}
