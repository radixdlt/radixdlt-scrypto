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
    containers: HashMap<ResourceAddress, Rc<RefCell<ResourceContainer>>>,
}

impl Worktop {
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), ResourceContainerError> {
        let resource_address = other.resource_address();
        let other_container = other.into_container()?;
        if let Some(mut container) = self.borrow_container_mut(resource_address) {
            return container.put(other_container);
        }
        self.put_container(resource_address, other_container);
        Ok(())
    }

    pub fn take(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> Result<Option<Bucket>, ResourceContainerError> {
        if let Some(mut container) = self.borrow_container_mut(resource_address) {
            container
                .take_by_amount(amount)
                .map(Bucket::new)
                .map(Option::Some)
        } else if !amount.is_zero() {
            Err(ResourceContainerError::InsufficientBalance)
        } else {
            Ok(None)
        }
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Result<Option<Bucket>, ResourceContainerError> {
        if let Some(mut container) = self.borrow_container_mut(resource_address) {
            container
                .take_by_ids(ids)
                .map(Bucket::new)
                .map(Option::Some)
        } else if !ids.is_empty() {
            Err(ResourceContainerError::InsufficientBalance)
        } else {
            Ok(None)
        }
    }

    pub fn take_all(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<Option<Bucket>, ResourceContainerError> {
        if let Some(mut container) = self.borrow_container_mut(resource_address) {
            Ok(Some(Bucket::new(container.take_all_liquid()?)))
        } else {
            Ok(None)
        }
    }

    pub fn resource_addresses(&self) -> Vec<ResourceAddress> {
        self.containers.keys().cloned().collect()
    }

    pub fn total_amount(&self, resource_address: ResourceAddress) -> Decimal {
        if let Some(container) = self.borrow_container(resource_address) {
            container.total_amount()
        } else {
            Decimal::zero()
        }
    }

    pub fn total_ids(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<BTreeSet<NonFungibleId>, ResourceContainerError> {
        if let Some(container) = self.borrow_container(resource_address) {
            container.total_ids()
        } else {
            Ok(BTreeSet::new())
        }
    }

    pub fn is_locked(&self) -> bool {
        for resource_address in self.resource_addresses() {
            if let Some(container) = self.borrow_container(resource_address) {
                if container.is_locked() {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        for resource_address in self.resource_addresses() {
            if let Some(container) = self.borrow_container(resource_address) {
                if !container.total_amount().is_zero() {
                    return false;
                }
            }
        }
        true
    }

    pub fn create_reference_for_proof(
        &self,
        resource_address: ResourceAddress,
    ) -> Option<Rc<RefCell<ResourceContainer>>> {
        self.containers.get(&resource_address).map(Clone::clone)
    }

    fn borrow_container(
        &self,
        resource_address: ResourceAddress,
    ) -> Option<Ref<ResourceContainer>> {
        self.containers.get(&resource_address).map(|c| c.borrow())
    }

    fn borrow_container_mut(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Option<RefMut<ResourceContainer>> {
        self.containers
            .get(&resource_address)
            .map(|c| c.borrow_mut())
    }

    // Note that this method overwrites existing container if any
    fn put_container(&mut self, resource_address: ResourceAddress, container: ResourceContainer) {
        self.containers
            .insert(resource_address, Rc::new(RefCell::new(container)));
    }
}
