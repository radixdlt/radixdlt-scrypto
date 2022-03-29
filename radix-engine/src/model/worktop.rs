use scrypto::engine::types::*;
use scrypto::rust::cell::{Ref, RefCell, RefMut};
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec::Vec;

use crate::model::{Bucket, ResourceContainer, ResourceContainerError};

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

    pub fn put(&mut self, other: Bucket) -> Result<(), ResourceContainerError> {
        let resource_def_id = other.resource_def_id();
        let other_container = other.into_container()?;
        if let Some(mut container) = self.borrow_container_mut(resource_def_id) {
            return container.put(other_container);
        }
        self.put_container(resource_def_id, other_container);
        Ok(())
    }

    pub fn take(
        &mut self,
        amount: Decimal,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, ResourceContainerError> {
        if let Some(mut container) = self.borrow_container_mut(resource_def_id) {
            container.take_by_amount(amount).map(Bucket::new)
        } else {
            Err(ResourceContainerError::InsufficientBalance)
        }
    }

    pub fn take_non_fungible(
        &mut self,
        id: &NonFungibleId,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, ResourceContainerError> {
        self.take_non_fungibles(&BTreeSet::from([id.clone()]), resource_def_id)
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    ) -> Result<Bucket, ResourceContainerError> {
        if let Some(mut container) = self.borrow_container_mut(resource_def_id) {
            container.take_by_ids(ids).map(Bucket::new)
        } else {
            Err(ResourceContainerError::InsufficientBalance)
        }
    }

    pub fn take_all(
        &mut self,
        resource_def_id: ResourceDefId,
    ) -> Result<Option<Bucket>, ResourceContainerError> {
        if let Some(mut container) = self.borrow_container_mut(resource_def_id) {
            Ok(Some(Bucket::new(container.take_all_liquid()?)))
        } else {
            Ok(None)
        }
    }

    pub fn resource_def_ids(&self) -> Vec<ResourceDefId> {
        self.containers.keys().cloned().collect()
    }

    pub fn total_amount(&self, resource_def_id: ResourceDefId) -> Decimal {
        if let Some(container) = self.borrow_container(resource_def_id) {
            container.total_amount()
        } else {
            Decimal::zero()
        }
    }

    pub fn total_ids(
        &self,
        resource_def_id: ResourceDefId,
    ) -> Result<BTreeSet<NonFungibleId>, ResourceContainerError> {
        if let Some(container) = self.borrow_container(resource_def_id) {
            container.total_ids()
        } else {
            Ok(BTreeSet::new())
        }
    }

    pub fn is_locked(&self) -> bool {
        for resource_def_id in self.resource_def_ids() {
            if let Some(container) = self.borrow_container(resource_def_id) {
                if container.is_locked() {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        for resource_def_id in self.resource_def_ids() {
            if let Some(container) = self.borrow_container(resource_def_id) {
                if !container.total_amount().is_zero() {
                    return false;
                }
            }
        }
        true
    }

    pub fn create_reference_for_proof(
        &self,
        resource_def_id: ResourceDefId,
    ) -> Option<Rc<RefCell<ResourceContainer>>> {
        self.containers.get(&resource_def_id).map(Clone::clone)
    }

    fn borrow_container(&self, resource_def_id: ResourceDefId) -> Option<Ref<ResourceContainer>> {
        self.containers.get(&resource_def_id).map(|c| c.borrow())
    }

    fn borrow_container_mut(
        &mut self,
        resource_def_id: ResourceDefId,
    ) -> Option<RefMut<ResourceContainer>> {
        self.containers
            .get(&resource_def_id)
            .map(|c| c.borrow_mut())
    }

    // Note that this method overwrites existing container if any
    fn put_container(&mut self, resource_def_id: ResourceDefId, container: ResourceContainer) {
        self.containers
            .insert(resource_def_id, Rc::new(RefCell::new(container)));
    }
}
