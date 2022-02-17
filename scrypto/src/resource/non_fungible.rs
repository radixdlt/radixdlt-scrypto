use crate::resource::*;
use crate::rust::marker::PhantomData;

/// Represents a non-fungible unit.
#[derive(Debug)]
pub struct NonFungible<T: NonFungibleData> {
    resource_def_id: ResourceDefId,
    key: NonFungibleKey,
    data: PhantomData<T>,
}

impl<T: NonFungibleData> From<(ResourceDefId, NonFungibleKey)> for NonFungible<T> {
    fn from(tuple: (ResourceDefId, NonFungibleKey)) -> Self {
        Self {
            resource_def_id: tuple.0,
            key: tuple.1.clone(),
            data: PhantomData,
        }
    }
}

impl<T: NonFungibleData> NonFungible<T> {
    /// Returns the resource definition.
    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    /// Returns the non-fungible ID.
    pub fn key(&self) -> NonFungibleKey {
        self.key.clone()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        self.resource_def_id().get_non_fungible_data(&self.key)
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T, auth: Proof) {
        self.resource_def_id()
            .update_non_fungible_data(&self.key, new_data, auth);
    }
}
