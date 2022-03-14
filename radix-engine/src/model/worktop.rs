use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec::Vec;

use crate::model::Bucket;
use crate::model::{ResourceContainer, ResourceError};

/// Represents an error when accessing a Worktop.
#[derive(Debug, Clone)]
pub enum WorktopError {
    ResourceError(ResourceError),
    ResourceContainerLocked,
    OtherBucketLocked,
}

/// Worktop collects resources from function or method returns.
#[derive(Debug)]
pub struct Worktop {
    containers: HashMap<ResourceDefId, Rc<ResourceContainer>>,
}

impl Worktop {
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), WorktopError> {
        let resource_def_id = other.resource_def_id();
        let other_container = other
            .take_container()
            .map_err(|_| WorktopError::OtherBucketLocked)?;

        if let Some(container) = self.containers.get_mut(&resource_def_id) {
            Rc::get_mut(container)
                .ok_or(WorktopError::ResourceContainerLocked)?
                .put(other_container)
                .map_err(WorktopError::ResourceError)
        } else {
            self.containers
                .insert(resource_def_id, Rc::new(other_container));
            Ok(())
        }
    }

    pub fn take(
        &mut self,
        amount: Decimal,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, WorktopError> {
        if let Some(container) = self.containers.get_mut(&resource_def_id) {
            Rc::get_mut(container)
                .ok_or(WorktopError::ResourceContainerLocked)?
                .take(amount)
                .map(Bucket::new)
        } else {
            Err(ResourceError::InsufficientBalance)
        }
        .map_err(WorktopError::ResourceError)
    }

    pub fn take_non_fungible(
        &mut self,
        key: &NonFungibleId,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, WorktopError> {
        self.take_non_fungibles(&BTreeSet::from([key.clone()]), resource_def_id)
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, WorktopError> {
        if let Some(container) = self.containers.get_mut(&resource_def_id) {
            Rc::get_mut(container)
                .ok_or(WorktopError::ResourceContainerLocked)?
                .take_non_fungibles(ids)
                .map(Bucket::new)
        } else {
            Err(ResourceError::InsufficientBalance)
        }
        .map_err(WorktopError::ResourceError)
    }

    pub fn take_all(&mut self, resource_def_id: ResourceDefId) -> Result<Bucket, WorktopError> {
        if let Some(container) = self.containers.remove(&resource_def_id) {
            Ok(Bucket::new(
                Rc::try_unwrap(container).map_err(|_| WorktopError::ResourceContainerLocked)?,
            ))
        } else {
            // TODO: a better approach would be to return an empty bucket
            Err(WorktopError::ResourceError(
                ResourceError::InsufficientBalance,
            ))
        }
    }

    pub fn resource_def_ids(&self) -> Vec<ResourceDefId> {
        let mut result = Vec::new();
        for (id, container) in &self.containers {
            // This is to make implementation agnostic.
            if container.liquid_amount().quantity() > 0.into() {
                result.push(*id);
            }
        }
        result
    }

    pub fn contains(&self, amount: Decimal, resource_def_id: ResourceDefId) -> bool {
        if let Some(container) = self.containers.get(&resource_def_id) {
            container.liquid_amount().quantity() >= amount
        } else {
            false
        }
    }
}
