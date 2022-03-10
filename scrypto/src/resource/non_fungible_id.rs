use sbor::*;

use crate::resource::*;

/// Identifier for a non-fungible unit.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub struct NonFungibleId {
    resource_def_id: ResourceDefId,
    key: NonFungibleKey
}

impl NonFungibleId {
    pub fn new(resource_def_id: ResourceDefId, key: NonFungibleKey) -> Self {
        Self {
            resource_def_id,
            key
        }
    }

    /// Returns the resource definition.
    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    /// Returns the non-fungible key.
    pub fn key(&self) -> NonFungibleKey {
        self.key.clone()
    }
}