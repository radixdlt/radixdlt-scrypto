use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::cell::{Ref, RefCell, RefMut};
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;

use crate::model::{Bucket, BucketError};
use crate::model::{ResourceContainer, ResourceContainerError};

/// Represents an error when accessing a vault.
#[derive(Debug, Clone, PartialEq)]
pub enum VaultError {
    ResourceContainerError(ResourceContainerError),
    BucketError(BucketError),
}

/// A persistent resource container.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct Vault {
    container: Rc<RefCell<ResourceContainer>>,
}

impl Vault {
    pub fn new(container: ResourceContainer) -> Self {
        Self {
            container: Rc::new(RefCell::new(container)),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), VaultError> {
        self.borrow_container_mut()
            .put(other.into_container().map_err(VaultError::BucketError)?)
            .map_err(VaultError::ResourceContainerError)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Bucket, VaultError> {
        Ok(Bucket::new(
            self.borrow_container_mut()
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
            self.borrow_container_mut()
                .take_non_fungibles(ids)
                .map_err(VaultError::ResourceContainerError)?,
        ))
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

    pub fn borrow_container(&self) -> Ref<ResourceContainer> {
        self.container.borrow()
    }

    pub fn borrow_container_mut(&mut self) -> RefMut<ResourceContainer> {
        self.container.borrow_mut()
    }

    pub fn refer_container(&self) -> Rc<RefCell<ResourceContainer>> {
        self.container.clone()
    }
}
