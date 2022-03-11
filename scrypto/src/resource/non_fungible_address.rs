use sbor::*;

use crate::resource::*;

/// Identifier for a non-fungible unit.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub struct NonFungibleAddress {
    resource_def_id: ResourceDefId,
    key: NonFungibleId,
}

impl NonFungibleAddress {
    pub fn new(resource_def_id: ResourceDefId, key: NonFungibleId) -> Self {
        Self {
            resource_def_id,
            key,
        }
    }

    /// Returns the resource definition.
    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    /// Returns the non-fungible key.
    pub fn key(&self) -> NonFungibleId {
        self.key.clone()
    }
}
