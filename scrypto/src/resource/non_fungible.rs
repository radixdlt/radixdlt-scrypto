use crate::resource::*;
use crate::rust::marker::PhantomData;

/// Represents a non-fungible unit.
#[derive(Debug)]
pub struct NonFungible<T: NonFungibleData> {
    resource_def_ref: ResourceDefRef,
    key: NonFungibleKey,
    data: PhantomData<T>,
}

impl<T: NonFungibleData> From<(ResourceDefRef, NonFungibleKey)> for NonFungible<T> {
    fn from(tuple: (ResourceDefRef, NonFungibleKey)) -> Self {
        Self {
            resource_def_ref: tuple.0,
            key: tuple.1.clone(),
            data: PhantomData,
        }
    }
}

impl<T: NonFungibleData> NonFungible<T> {
    /// Returns the resource definition.
    pub fn resource_def_ref(&self) -> ResourceDefRef {
        self.resource_def_ref
    }

    /// Returns the non-fungible ID.
    pub fn key(&self) -> NonFungibleKey {
        self.key.clone()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        self.resource_def_ref().get_non_fungible_data(&self.key)
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T, auth: BucketRef) {
        self.resource_def_ref()
            .update_non_fungible_data(&self.key, new_data, auth);
    }
}
