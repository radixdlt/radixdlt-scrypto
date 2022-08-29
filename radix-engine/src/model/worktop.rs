use crate::engine::{DropFailure, HeapRENode, InvokeError, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{Bucket, ResourceContainer, ResourceContainerError};
use crate::types::*;
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
    // TODO: refactor worktop to be `HashMap<ResourceAddress, BucketId>`
    containers: HashMap<ResourceAddress, Rc<RefCell<ResourceContainer>>>,
}

#[derive(Debug)]
pub enum WorktopError {
    InvalidRequestData(DecodeError),
    MethodNotFound(String),
    ResourceContainerError(ResourceContainerError),
    ResourceNotFound(ResourceAddress),
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

    pub fn drop(self) -> Result<(), DropFailure> {
        for (_address, container) in self.containers {
            if !container.borrow().is_empty() {
                return Err(DropFailure::Worktop);
            }
        }

        Ok(())
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

    pub fn main<'s, Y, W, I, R>(
        worktop_fn: WorktopFnIdentifier,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<WorktopError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let mut node_ref = system_api
            .substate_borrow_mut(&SubstateId::Worktop)
            .map_err(InvokeError::downstream)?;
        let worktop = node_ref.worktop();

        let rtn = match worktop_fn {
            WorktopFnIdentifier::Put => {
                let input: WorktopPutInput =
                    scrypto_decode(&args.raw).map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let bucket = system_api
                    .node_drop(&RENodeId::Bucket(input.bucket.0))
                    .map_err(|e| InvokeError::Downstream(e))?
                    .into();
                worktop
                    .put(bucket)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceContainerError(e)))?;
                Ok(ScryptoValue::from_typed(&()))
            }
            WorktopFnIdentifier::TakeAmount => {
                let input: WorktopTakeAmountInput =
                    scrypto_decode(&args.raw).map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let maybe_container = worktop
                    .take(input.amount, input.resource_address)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceContainerError(e)))?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    let resource_type = {
                        let node_ref = system_api
                            .borrow_node(&RENodeId::ResourceManager(input.resource_address))
                            .map_err(|e| InvokeError::Downstream(e))?;
                        let resource_manager = node_ref.resource_manager();
                        resource_manager.resource_type()
                    };

                    ResourceContainer::new_empty(input.resource_address, resource_type)
                };
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(resource_container)))
                    .map_err(|e| InvokeError::Downstream(e))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopFnIdentifier::TakeAll => {
                let input: WorktopTakeAllInput =
                    scrypto_decode(&args.raw).map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let maybe_container = worktop
                    .take_all(input.resource_address)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceContainerError(e)))?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    let resource_type = {
                        let node_ref = system_api
                            .borrow_node(&RENodeId::ResourceManager(input.resource_address))
                            .map_err(|e| InvokeError::Downstream(e))?;
                        let resource_manager = node_ref.resource_manager();
                        resource_manager.resource_type()
                    };

                    ResourceContainer::new_empty(input.resource_address, resource_type)
                };

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(resource_container)))
                    .map_err(|e| InvokeError::Downstream(e))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopFnIdentifier::TakeNonFungibles => {
                let input: WorktopTakeNonFungiblesInput =
                    scrypto_decode(&args.raw).map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let maybe_container = worktop
                    .take_non_fungibles(&input.ids, input.resource_address)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceContainerError(e)))?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    let resource_type = {
                        let node_ref = system_api
                            .borrow_node(&RENodeId::ResourceManager(input.resource_address))
                            .map_err(|e| InvokeError::Downstream(e))?;
                        let resource_manager = node_ref.resource_manager();
                        resource_manager.resource_type()
                    };

                    ResourceContainer::new_empty(input.resource_address, resource_type)
                };

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(resource_container)))
                    .map_err(|e| InvokeError::Downstream(e))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopFnIdentifier::AssertContains => {
                let input: WorktopAssertContainsInput =
                    scrypto_decode(&args.raw).map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                if worktop.total_amount(input.resource_address).is_zero() {
                    Err(InvokeError::Error(WorktopError::AssertionFailed))
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            WorktopFnIdentifier::AssertContainsAmount => {
                let input: WorktopAssertContainsAmountInput =
                    scrypto_decode(&args.raw).map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                if worktop.total_amount(input.resource_address) < input.amount {
                    Err(InvokeError::Error(WorktopError::AssertionFailed))
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            WorktopFnIdentifier::AssertContainsNonFungibles => {
                let input: WorktopAssertContainsNonFungiblesInput =
                    scrypto_decode(&args.raw).map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                if !worktop
                    .total_ids(input.resource_address)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceContainerError(e)))?
                    .is_superset(&input.ids)
                {
                    Err(InvokeError::Error(WorktopError::AssertionFailed))
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            WorktopFnIdentifier::Drain => {
                let _: WorktopDrainInput =
                    scrypto_decode(&args.raw).map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let mut buckets = Vec::new();
                for (_, container) in worktop.containers.drain() {
                    let container = container
                        .borrow_mut()
                        .take_all_liquid()
                        .map_err(|e| InvokeError::Error(WorktopError::ResourceContainerError(e)))?;
                    if !container.is_empty() {
                        let bucket_id = system_api
                            .node_create(HeapRENode::Bucket(Bucket::new(container)))
                            .map_err(|e| InvokeError::Downstream(e))?
                            .into();
                        buckets.push(scrypto::resource::Bucket(bucket_id));
                    }
                }
                Ok(ScryptoValue::from_typed(&buckets))
            }
        }?;

        system_api
            .substate_return_mut(node_ref)
            .map_err(|e| InvokeError::Downstream(e))?;

        Ok(rtn)
    }
}
