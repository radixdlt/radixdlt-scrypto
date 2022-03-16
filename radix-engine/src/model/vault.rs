use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;

use crate::model::Bucket;
use crate::model::{ResourceContainer, ResourceContainerError};

/// Represents an error when accessing a vault.
#[derive(Debug, Clone, PartialEq)]
pub enum VaultError {
    ResourceContainerError(ResourceContainerError),
    VaultLocked,
    OtherBucketLocked,
}

/// A persistent resource container.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct Vault {
    container: ResourceContainer,
}

impl Vault {
    pub fn new(container: ResourceContainer) -> Self {
        Self { container }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), VaultError> {
        self.container
            .put(other.into_container())
            .map_err(VaultError::ResourceContainerError)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Bucket, VaultError> {
        Ok(Bucket::new(
            self.container
                .take(amount)
                .map_err(VaultError::ResourceContainerError)?,
        ))
    }

    pub fn take_non_fungible(&mut self, id: &NonFungibleId) -> Result<Bucket, VaultError> {
        self.take_non_fungibles(&BTreeSet::from([id.clone()]))
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<Bucket, VaultError> {
        Ok(Bucket::new(
            self.container
                .take_non_fungibles(ids)
                .map_err(VaultError::ResourceContainerError)?,
        ))
    }

    pub fn liquid_amount(&self) -> Amount {
        self.container.liquid_amount()
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.container.resource_def_id()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.container.resource_type()
    }

    pub fn borrow_container(&mut self) -> &mut ResourceContainer {
        &mut self.container
    }
}
