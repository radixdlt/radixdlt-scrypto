use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::vec::Vec;

use crate::model::Bucket;
use crate::model::{ResourceContainer, ResourceContainerError};

/// Represents an error when accessing a Worktop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorktopError {
    ResourceContainerError(ResourceContainerError),
    ResourceContainerLocked,
    OtherBucketLocked,
}

/// Worktop collects resources from function or method returns.
#[derive(Debug)]
pub struct Worktop {
    containers: HashMap<ResourceDefId, ResourceContainer>,
}

impl Worktop {
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), WorktopError> {
        let resource_def_id = other.resource_def_id();
        if let Some(container) = self.borrow_container(resource_def_id) {
            container
                .put(other.into_container())
                .map_err(WorktopError::ResourceContainerError)
        } else {
            self.containers
                .insert(resource_def_id, other.into_container());
            Ok(())
        }
    }

    pub fn take(
        &mut self,
        amount: Decimal,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, WorktopError> {
        if let Some(container) = self.borrow_container(resource_def_id) {
            container.take(amount).map(Bucket::new)
        } else {
            Err(ResourceContainerError::InsufficientBalance)
        }
        .map_err(WorktopError::ResourceContainerError)
    }

    pub fn take_non_fungible(
        &mut self,
        id: &NonFungibleId,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, WorktopError> {
        self.take_non_fungibles(&BTreeSet::from([id.clone()]), resource_def_id)
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, WorktopError> {
        if let Some(container) = self.borrow_container(resource_def_id) {
            container.take_non_fungibles(ids).map(Bucket::new)
        } else {
            Err(ResourceContainerError::InsufficientBalance)
        }
        .map_err(WorktopError::ResourceContainerError)
    }

    pub fn take_all(&mut self, resource_def_id: ResourceDefId) -> Option<Bucket> {
        self.take_container(resource_def_id).map(Bucket::new)
    }

    pub fn resource_def_ids(&self) -> Vec<ResourceDefId> {
        self.containers.keys().cloned().collect()
    }

    pub fn contains(&self, amount: Decimal, resource_def_id: ResourceDefId) -> bool {
        if let Some(container) = self.containers.get(&resource_def_id) {
            container.liquid_amount().as_quantity() >= amount
        } else {
            false
        }
    }

    pub fn borrow_container(
        &mut self,
        resource_def_id: ResourceDefId,
    ) -> Option<&mut ResourceContainer> {
        self.containers.get_mut(&resource_def_id)
    }

    pub fn take_container(&mut self, resource_def_id: ResourceDefId) -> Option<ResourceContainer> {
        self.containers.remove(&resource_def_id)
    }
}
