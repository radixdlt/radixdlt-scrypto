use radix_engine_interface::model::*;
use sbor::rust::marker::PhantomData;

use crate::borrow_resource_manager;
use crate::resource::*;

pub trait ScryptoNonFungibleLocalId {
    /// Creates a non-fungible ID from some uuid.
    fn random() -> Self;
}

impl ScryptoNonFungibleLocalId for NonFungibleLocalId {
    fn random() -> Self {
        let uuid = crate::runtime::Runtime::generate_uuid();
        Self::uuid(uuid).unwrap()
    }
}

/// Represents a non-fungible unit.
#[derive(Debug)]
pub struct NonFungible<T: NonFungibleData> {
    non_fungible_global_id: NonFungibleGlobalId,
    data: PhantomData<T>,
}

impl<T: NonFungibleData> From<NonFungibleGlobalId> for NonFungible<T> {
    fn from(non_fungible_global_id: NonFungibleGlobalId) -> Self {
        Self {
            non_fungible_global_id,
            data: PhantomData,
        }
    }
}

impl<T: NonFungibleData> NonFungible<T> {
    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        self.non_fungible_global_id.resource_address().clone()
    }

    /// Returns a reference to the non-fungible address.
    pub fn global_id(&self) -> &NonFungibleGlobalId {
        &self.non_fungible_global_id
    }

    /// Returns a reference to the the non-fungible ID.
    pub fn local_id(&self) -> &NonFungibleLocalId {
        self.non_fungible_global_id.local_id()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        borrow_resource_manager!(self.resource_address().clone())
            .get_non_fungible_data(self.local_id())
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T) {
        borrow_resource_manager!(self.resource_address().clone())
            .update_non_fungible_data(self.local_id(), new_data);
    }
}
