use scrypto::engine::types::*;
use scrypto::rust::cell::{Ref, RefCell, RefMut};
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec::Vec;

use crate::model::{Bucket, BucketError};
use crate::model::{ResourceContainer, ResourceContainerError};

/// Represents an error when accessing a Worktop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorktopError {
    ResourceContainerError(ResourceContainerError),
    BucketError(BucketError),
    ResourceContainerLocked,
}

/// Worktop collects resources from function or method returns.
#[derive(Debug)]
pub struct Worktop {
    containers: HashMap<ResourceDefId, Rc<RefCell<ResourceContainer>>>,
}

impl Worktop {
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), WorktopError> {
        let resource_def_id = other.resource_def_id();
        let other_container = other.into_container().map_err(WorktopError::BucketError)?;
        if let Some(mut container) = self.borrow_container_mut(resource_def_id) {
            return container
                .put(other_container)
                .map_err(WorktopError::ResourceContainerError);
        }
        self.put_container(resource_def_id, other_container);
        Ok(())
    }

    pub fn take(
        &mut self,
        amount: Decimal,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, WorktopError> {
        if let Some(mut container) = self.borrow_container_mut(resource_def_id) {
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
        if let Some(mut container) = self.borrow_container_mut(resource_def_id) {
            container.take_non_fungibles(ids).map(Bucket::new)
        } else {
            Err(ResourceContainerError::InsufficientBalance)
        }
        .map_err(WorktopError::ResourceContainerError)
    }

    pub fn take_all(
        &mut self,
        resource_def_id: ResourceDefId,
    ) -> Result<Option<Bucket>, WorktopError> {
        self.take_container(resource_def_id)
            .map(|o| o.map(Bucket::new))
    }

    pub fn resource_def_ids(&self) -> Vec<ResourceDefId> {
        self.containers.keys().cloned().collect()
    }

    pub fn contains(&self, amount: Decimal, resource_def_id: ResourceDefId) -> bool {
        if let Some(container) = self.borrow_container(resource_def_id) {
            container.liquid_amount().as_quantity() >= amount
        } else {
            false
        }
    }

    pub fn borrow_container(
        &self,
        resource_def_id: ResourceDefId,
    ) -> Option<Ref<ResourceContainer>> {
        self.containers.get(&resource_def_id).map(|c| c.borrow())
    }

    pub fn borrow_container_mut(
        &mut self,
        resource_def_id: ResourceDefId,
    ) -> Option<RefMut<ResourceContainer>> {
        self.containers
            .get(&resource_def_id)
            .map(|c| c.borrow_mut())
    }

    pub fn refer_container(
        &self,
        resource_def_id: ResourceDefId,
    ) -> Option<Rc<RefCell<ResourceContainer>>> {
        self.containers.get(&resource_def_id).map(Clone::clone)
    }

    pub fn take_container(
        &mut self,
        resource_def_id: ResourceDefId,
    ) -> Result<Option<ResourceContainer>, WorktopError> {
        if let Some(c) = self.containers.remove(&resource_def_id) {
            Ok(Some(
                Rc::try_unwrap(c)
                    .map_err(|_| WorktopError::ResourceContainerLocked)?
                    .into_inner(),
            ))
        } else {
            Ok(None)
        }
    }

    // Note that this method overwrites existing container if any
    fn put_container(&mut self, resource_def_id: ResourceDefId, container: ResourceContainer) {
        self.containers
            .insert(resource_def_id, Rc::new(RefCell::new(container)));
    }
}
