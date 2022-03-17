use scrypto::engine::types::*;
use scrypto::prelude::NonFungibleAddress;
use scrypto::rust::cell::{Ref, RefCell, RefMut};
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;

use crate::model::{ResourceContainer, ResourceContainerError};

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BucketError {
    ResourceContainerError(ResourceContainerError),
    ResourceContainerLocked,
}

/// A transient resource container.
#[derive(Debug)]
pub struct Bucket {
    container: Rc<RefCell<ResourceContainer>>,
}

impl Bucket {
    pub fn new(container: ResourceContainer) -> Self {
        Self {
            container: Rc::new(RefCell::new(container)),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), BucketError> {
        self.borrow_container_mut()
            .put(other.into_container()?)
            .map_err(BucketError::ResourceContainerError)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Bucket, BucketError> {
        Ok(Bucket::new(
            self.borrow_container_mut()
                .take(amount)
                .map_err(BucketError::ResourceContainerError)?,
        ))
    }

    pub fn take_non_fungible(&mut self, id: &NonFungibleId) -> Result<Bucket, BucketError> {
        self.take_non_fungibles(&BTreeSet::from([id.clone()]))
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<Bucket, BucketError> {
        Ok(Bucket::new(
            self.borrow_container_mut()
                .take_non_fungibles(ids)
                .map_err(BucketError::ResourceContainerError)?,
        ))
    }

    pub fn contains_non_fungible_address(&self, non_fungible_address: &NonFungibleAddress) -> bool {
        if self.resource_def_id() != non_fungible_address.resource_def_id() {
            return false;
        }

        match self
            .borrow_container()
            .liquid_amount()
            .as_non_fungible_ids()
        {
            Err(_) => false,
            Ok(non_fungible_ids) => non_fungible_ids
                .iter()
                .any(|k| k.eq(&non_fungible_address.non_fungible_id())),
        }
    }

    pub fn liquid_amount(&self) -> Amount {
        self.borrow_container().liquid_amount()
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.borrow_container().resource_def_id()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.borrow_container().resource_type()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_container().is_locked()
    }

    pub fn borrow_container(&self) -> Ref<ResourceContainer> {
        self.container.borrow()
    }

    pub fn borrow_container_mut(&mut self) -> RefMut<ResourceContainer> {
        self.container.borrow_mut()
    }

    pub fn refer_container(&self) -> Rc<RefCell<ResourceContainer>> {
        self.container.clone()
    }

    pub fn into_container(self) -> Result<ResourceContainer, BucketError> {
        Rc::try_unwrap(self.container)
            .map_err(|_| BucketError::ResourceContainerLocked)
            .map(|c| c.into_inner())
    }
}
