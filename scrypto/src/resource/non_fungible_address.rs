use sbor::*;

use crate::resource::*;

/// Identifier for a non-fungible unit.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub struct NonFungibleAddress {
    resource_def_id: ResourceDefId,
    non_fungible_id: NonFungibleId,
}

impl NonFungibleAddress {
    pub fn new(resource_def_id: ResourceDefId, non_fungible_id: NonFungibleId) -> Self {
        Self {
            resource_def_id,
            non_fungible_id,
        }
    }

    /// Returns the resource definition.
    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    /// Returns the non-fungible id.
    pub fn non_fungible_id(&self) -> NonFungibleId {
        self.non_fungible_id.clone()
    }
}
