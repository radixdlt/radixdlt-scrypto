use sbor::rust::cell::{Ref, RefCell, RefMut};
use sbor::rust::collections::BTreeSet;
use sbor::rust::collections::HashMap;
use sbor::rust::rc::Rc;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;

use crate::engine::SystemApi;
use crate::ledger::ReadableSubstateStore;
use crate::model::WorktopError::InvalidMethod;
use crate::model::{Bucket, ResourceContainer, ResourceContainerError};
use crate::wasm::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopPutInput {
    pub bucket: scrypto::resource::Bucket,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeAmountInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeNonFungiblesInput {
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeAllInput {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsInput {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsAmountInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsNonFungiblesInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopDrainInput {}

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
    InvalidMethod,
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

    pub fn main<
        'p,
        't,
        's,
        Y: SystemApi<'p, 't, 's, W, I, S>,
        W: WasmEngine<I>,
        I: WasmInstance,
        S: 's + ReadableSubstateStore,
    >(
        &mut self,
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, WorktopError> {
        match method_name {
            "put" => {
                let input: WorktopPutInput =
                    scrypto_decode(&arg.raw).map_err(|e| WorktopError::InvalidRequestData(e))?;
                let bucket = system_api
                    .take_native_value(&ValueId::Transient(TransientValueId::Bucket(
                        input.bucket.0,
                    )))
                    .into();
                self.put(bucket)
                    .map_err(WorktopError::ResourceContainerError)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            "take_amount" => {
                let input: WorktopTakeAmountInput =
                    scrypto_decode(&arg.raw).map_err(|e| WorktopError::InvalidRequestData(e))?;
                let maybe_container = self
                    .take(input.amount, input.resource_address)
                    .map_err(WorktopError::ResourceContainerError)?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    let resource_type = {
                        let value = system_api.borrow_value(&ValueId::Resource(input.resource_address));
                        let resource_manager = value.resource_manager();
                        resource_manager.resource_type()
                    };

                    ResourceContainer::new_empty(input.resource_address, resource_type)
                };
                let bucket_id = system_api
                    .native_create(Bucket::new(resource_container))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "take_all" => {
                let input: WorktopTakeAllInput =
                    scrypto_decode(&arg.raw).map_err(|e| WorktopError::InvalidRequestData(e))?;
                let maybe_container = self
                    .take_all(input.resource_address)
                    .map_err(WorktopError::ResourceContainerError)?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    let resource_type = {
                        let value = system_api.borrow_value(&ValueId::Resource(input.resource_address));
                        let resource_manager = value.resource_manager();
                        resource_manager.resource_type()
                    };

                    ResourceContainer::new_empty(input.resource_address, resource_type)
                };

                let bucket_id = system_api
                    .native_create(Bucket::new(resource_container))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "take_non_fungibles" => {
                let input: WorktopTakeNonFungiblesInput =
                    scrypto_decode(&arg.raw).map_err(|e| WorktopError::InvalidRequestData(e))?;
                let maybe_container = self
                    .take_non_fungibles(&input.ids, input.resource_address)
                    .map_err(WorktopError::ResourceContainerError)?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    let resource_type = {
                        let value = system_api.borrow_value(&ValueId::Resource(input.resource_address));
                        let resource_manager = value.resource_manager();
                        resource_manager.resource_type()
                    };

                    ResourceContainer::new_empty(input.resource_address, resource_type)
                };

                let bucket_id = system_api
                    .native_create(Bucket::new(resource_container))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "assert_contains" => {
                let input: WorktopAssertContainsInput =
                    scrypto_decode(&arg.raw).map_err(|e| WorktopError::InvalidRequestData(e))?;
                if self.total_amount(input.resource_address).is_zero() {
                    Err(WorktopError::AssertionFailed)
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            "assert_contains_amount" => {
                let input: WorktopAssertContainsAmountInput =
                    scrypto_decode(&arg.raw).map_err(|e| WorktopError::InvalidRequestData(e))?;
                if self.total_amount(input.resource_address) < input.amount {
                    Err(WorktopError::AssertionFailed)
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            "assert_contains_non_fungibles" => {
                let input: WorktopAssertContainsNonFungiblesInput =
                    scrypto_decode(&arg.raw).map_err(|e| WorktopError::InvalidRequestData(e))?;
                if !self
                    .total_ids(input.resource_address)
                    .map_err(WorktopError::ResourceContainerError)?
                    .is_superset(&input.ids)
                {
                    Err(WorktopError::AssertionFailed)
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            "drain" => {
                let _: WorktopDrainInput =
                    scrypto_decode(&arg.raw).map_err(|e| WorktopError::InvalidRequestData(e))?;
                let mut buckets = Vec::new();
                for (_, container) in self.containers.drain() {
                    let container = container
                        .borrow_mut()
                        .take_all_liquid()
                        .map_err(WorktopError::ResourceContainerError)?;
                    if !container.is_empty() {
                        let bucket_id = system_api
                            .native_create(Bucket::new(container))
                            .unwrap()
                            .into();
                        buckets.push(scrypto::resource::Bucket(bucket_id));
                    }
                }
                Ok(ScryptoValue::from_typed(&buckets))
            }
            _ => Err(InvalidMethod),
        }
    }
}
