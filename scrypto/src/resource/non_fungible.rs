use crate::resource::*;
use crate::rust::marker::PhantomData;

/// Represents a non-fungible unit.
#[derive(Debug)]
pub struct NonFungible<T: NonFungibleData> {
    resource_def: ResourceDef,
    key: NonFungibleKey,
    data: PhantomData<T>,
}

impl<T: NonFungibleData> From<(ResourceDef, NonFungibleKey)> for NonFungible<T> {
    fn from(tuple: (ResourceDef, NonFungibleKey)) -> Self {
        Self {
            resource_def: tuple.0,
            key: tuple.1.clone(),
            data: PhantomData,
        }
    }
}

impl<T: NonFungibleData> NonFungible<T> {
    /// Returns the resource definition.
    pub fn resource_def(&self) -> ResourceDef {
        self.resource_def
    }

    /// Returns the non-fungible ID.
    pub fn key(&self) -> NonFungibleKey {
        self.key.clone()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        self.resource_def().get_non_fungible_data(&self.key)
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T, auth: BucketRef) {
        self.resource_def()
            .update_non_fungible_data(&self.key, new_data, auth);
    }
}
