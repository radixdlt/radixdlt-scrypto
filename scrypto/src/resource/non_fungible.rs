use crate::resource::*;
use crate::resource_def;
use crate::rust::marker::PhantomData;

/// Represents a non-fungible unit.
#[derive(Debug)]
pub struct NonFungible<T: NonFungibleData> {
    id: NonFungibleId,
    data: PhantomData<T>,
}


impl<T: NonFungibleData> From<NonFungibleId> for NonFungible<T> {
    fn from(id: NonFungibleId) -> Self {
        Self {
            id,
            data: PhantomData,
        }
    }
}

impl<T: NonFungibleData> NonFungible<T> {
    /// Returns the resource definition.
    pub fn resource_def_id(&self) -> ResourceDefId {
        self.id.resource_def_id()
    }

    /// Returns the non-fungible ID.
    pub fn key(&self) -> NonFungibleKey {
        self.id.key()
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
