use crate::engine::SystemApi;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::types::*;
use scrypto::rust::cell::{Ref, RefCell, RefMut};
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::String;
use scrypto::values::ScryptoValue;

use crate::model::{Bucket, ResourceContainer, ResourceContainerError, ResourceManager};

#[derive(Debug, TypeId, Encode, Decode)]
pub enum WorktopMethod {
    Put(scrypto::resource::Bucket),
    TakeAmount(Decimal, ResourceAddress),
    TakeAll(ResourceAddress),
    TakeNonFungibles(BTreeSet<NonFungibleId>, ResourceAddress),
    AssertContains(ResourceAddress),
    AssertContainsAmount(Decimal, ResourceAddress),
    AssertContainsNonFungibles(BTreeSet<NonFungibleId>, ResourceAddress),
    Drain(),
}

/// Worktop collects resources from function or method returns.
#[derive(Debug)]
pub struct Worktop {
    containers: HashMap<ResourceAddress, Rc<RefCell<ResourceContainer>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorktopError {
    InvalidRequestData(DecodeError),
    MethodNotFound(String),
    ResourceContainerError(ResourceContainerError),
    ResourceDoesNotExist(ResourceAddress),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    AssertionFailed,
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

    fn take(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> Result<Option<ResourceContainer>, ResourceContainerError> {
        if let Some(mut container) = self.borrow_container_mut(resource_address) {
            container.take_by_amount(amount).map(Option::Some)
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
    ) -> Result<Option<ResourceContainer>, ResourceContainerError> {
        if let Some(mut container) = self.borrow_container_mut(resource_address) {
            container.take_by_ids(ids).map(Option::Some)
        } else if !ids.is_empty() {
            Err(ResourceContainerError::InsufficientBalance)
        } else {
            Ok(None)
        }
    }

    fn take_all(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<Option<ResourceContainer>, ResourceContainerError> {
        if let Some(mut container) = self.borrow_container_mut(resource_address) {
            Ok(Some(container.take_all_liquid()?))
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

    pub fn main<S: SystemApi>(
        &mut self,
        arg: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, WorktopError> {
        let method: WorktopMethod =
            scrypto_decode(&arg.raw).map_err(|e| WorktopError::InvalidRequestData(e))?;

        match method {
            WorktopMethod::Put(bucket) => {
                let bucket = system_api
                    .take_bucket(bucket.0)
                    .map_err(|_| WorktopError::CouldNotTakeBucket)?;
                self.put(bucket)
                    .map_err(WorktopError::ResourceContainerError)?;
                Ok(ScryptoValue::from_value(&()))
            }
            WorktopMethod::TakeAmount(amount, resource_address) => {
                let maybe_container = self
                    .take(amount, resource_address)
                    .map_err(WorktopError::ResourceContainerError)?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    let resource_manager: ResourceManager = system_api
                        .borrow_global_mut_resource_manager(resource_address)
                        .map_err(|_| WorktopError::ResourceDoesNotExist(resource_address))?;
                    let resource_type = resource_manager.resource_type();
                    system_api.return_borrowed_global_resource_manager(
                        resource_address,
                        resource_manager,
                    );
                    ResourceContainer::new_empty(resource_address, resource_type)
                };

                let bucket_id = system_api
                    .create_bucket(resource_container)
                    .map_err(|_| WorktopError::CouldNotCreateBucket)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopMethod::TakeAll(resource_address) => {
                let maybe_container = self
                    .take_all(resource_address)
                    .map_err(WorktopError::ResourceContainerError)?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    let resource_manager: ResourceManager = system_api
                        .borrow_global_mut_resource_manager(resource_address)
                        .map_err(|_| WorktopError::ResourceDoesNotExist(resource_address))?;
                    let resource_type = resource_manager.resource_type();
                    system_api.return_borrowed_global_resource_manager(
                        resource_address,
                        resource_manager,
                    );
                    ResourceContainer::new_empty(resource_address, resource_type)
                };

                let bucket_id = system_api
                    .create_bucket(resource_container)
                    .map_err(|_| WorktopError::CouldNotCreateBucket)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopMethod::TakeNonFungibles(non_fungible_ids, resource_address) => {
                let maybe_container = self
                    .take_non_fungibles(&non_fungible_ids, resource_address)
                    .map_err(WorktopError::ResourceContainerError)?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    let resource_manager: ResourceManager = system_api
                        .borrow_global_mut_resource_manager(resource_address)
                        .map_err(|_| WorktopError::ResourceDoesNotExist(resource_address))?;
                    let resource_type = resource_manager.resource_type();
                    system_api.return_borrowed_global_resource_manager(
                        resource_address,
                        resource_manager,
                    );
                    ResourceContainer::new_empty(resource_address, resource_type)
                };

                let bucket_id = system_api
                    .create_bucket(resource_container)
                    .map_err(|_| WorktopError::CouldNotCreateBucket)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopMethod::AssertContains(resource_address) => {
                if self.total_amount(resource_address).is_zero() {
                    Err(WorktopError::AssertionFailed)
                } else {
                    Ok(ScryptoValue::from_value(&()))
                }
            }
            WorktopMethod::AssertContainsAmount(amount, resource_address) => {
                if self.total_amount(resource_address) < amount {
                    Err(WorktopError::AssertionFailed)
                } else {
                    Ok(ScryptoValue::from_value(&()))
                }
            }
            WorktopMethod::AssertContainsNonFungibles(ids, resource_address) => {
                if !self
                    .total_ids(resource_address)
                    .map_err(WorktopError::ResourceContainerError)?
                    .is_superset(&ids)
                {
                    Err(WorktopError::AssertionFailed)
                } else {
                    Ok(ScryptoValue::from_value(&()))
                }
            }
            WorktopMethod::Drain() => {
                let mut buckets = Vec::new();
                for (_, container) in self.containers.drain() {
                    let container = container
                        .borrow_mut()
                        .take_all_liquid()
                        .map_err(WorktopError::ResourceContainerError)?;
                    if !container.is_empty() {
                        let bucket_id = system_api
                            .create_bucket(container)
                            .map_err(|_| WorktopError::CouldNotCreateBucket)?;
                        buckets.push(scrypto::resource::Bucket(bucket_id));
                    }
                }
                Ok(ScryptoValue::from_value(&buckets))
            }
        }
    }
}
