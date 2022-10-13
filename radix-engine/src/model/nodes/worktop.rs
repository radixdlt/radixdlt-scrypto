use crate::engine::{DropFailure, HeapRENode, InvokeError, LockFlags, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{BucketSubstate, LockableResource, Resource, ResourceOperationError};
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
pub struct WorktopSubstate {
    // TODO: refactor worktop to be `HashMap<ResourceAddress, BucketId>`
    resources: HashMap<ResourceAddress, Rc<RefCell<LockableResource>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum WorktopError {
    InvalidRequestData(DecodeError),
    MethodNotFound(String),
    ResourceOperationError(ResourceOperationError),
    ResourceNotFound(ResourceAddress),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    AssertionFailed,
}

impl WorktopSubstate {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn drop(self) -> Result<(), DropFailure> {
        for (_address, resource) in self.resources {
            if !resource.borrow().is_empty() {
                return Err(DropFailure::Worktop);
            }
        }

        Ok(())
    }

    pub fn put(&mut self, other: BucketSubstate) -> Result<(), ResourceOperationError> {
        let resource_address = other.resource_address();
        let other_resource = other.resource()?;
        if let Some(mut resource) = self.borrow_resource_mut(resource_address) {
            return resource.put(other_resource);
        }
        self.resources.insert(
            resource_address,
            Rc::new(RefCell::new(other_resource.into())),
        );
        Ok(())
    }

    fn take(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> Result<Option<Resource>, ResourceOperationError> {
        if let Some(mut resource) = self.borrow_resource_mut(resource_address) {
            resource.take_by_amount(amount).map(Option::Some)
        } else if !amount.is_zero() {
            Err(ResourceOperationError::InsufficientBalance)
        } else {
            Ok(None)
        }
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Result<Option<Resource>, ResourceOperationError> {
        if let Some(mut resource) = self.borrow_resource_mut(resource_address) {
            resource.take_by_ids(ids).map(Option::Some)
        } else if !ids.is_empty() {
            Err(ResourceOperationError::InsufficientBalance)
        } else {
            Ok(None)
        }
    }

    fn take_all(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<Option<Resource>, ResourceOperationError> {
        if let Some(mut resource) = self.borrow_resource_mut(resource_address) {
            Ok(Some(resource.take_all_liquid()?))
        } else {
            Ok(None)
        }
    }

    pub fn resource_addresses(&self) -> Vec<ResourceAddress> {
        self.resources.keys().cloned().collect()
    }

    pub fn total_amount(&self, resource_address: ResourceAddress) -> Decimal {
        if let Some(resource) = self.borrow_resource(resource_address) {
            resource.total_amount()
        } else {
            Decimal::zero()
        }
    }

    pub fn total_ids(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<BTreeSet<NonFungibleId>, ResourceOperationError> {
        if let Some(resource) = self.borrow_resource(resource_address) {
            resource.total_ids()
        } else {
            Ok(BTreeSet::new())
        }
    }

    pub fn is_locked(&self) -> bool {
        for resource_address in self.resource_addresses() {
            if let Some(resource) = self.borrow_resource(resource_address) {
                if resource.is_locked() {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        for resource_address in self.resource_addresses() {
            if let Some(resource) = self.borrow_resource(resource_address) {
                if !resource.total_amount().is_zero() {
                    return false;
                }
            }
        }
        true
    }

    pub fn create_reference_for_proof(
        &self,
        resource_address: ResourceAddress,
    ) -> Option<Rc<RefCell<LockableResource>>> {
        self.resources.get(&resource_address).map(Clone::clone)
    }

    fn borrow_resource(&self, resource_address: ResourceAddress) -> Option<Ref<LockableResource>> {
        self.resources.get(&resource_address).map(|c| c.borrow())
    }

    fn borrow_resource_mut(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Option<RefMut<LockableResource>> {
        self.resources
            .get(&resource_address)
            .map(|c| c.borrow_mut())
    }

    pub fn method_locks(method: WorktopMethod) -> LockFlags {
        match method {
            WorktopMethod::TakeAll => LockFlags::MUTABLE,
            WorktopMethod::TakeAmount => LockFlags::MUTABLE,
            WorktopMethod::TakeNonFungibles => LockFlags::MUTABLE,
            WorktopMethod::Put => LockFlags::MUTABLE,
            WorktopMethod::AssertContains => LockFlags::read_only(),
            WorktopMethod::AssertContainsAmount => LockFlags::read_only(),
            WorktopMethod::AssertContainsNonFungibles => LockFlags::read_only(),
            WorktopMethod::Drain => LockFlags::MUTABLE,
        }
    }

    pub fn main<'s, Y, W, I, R>(
        method: WorktopMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<WorktopError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle =
            system_api.lock_substate(node_id, offset, Self::method_locks(method))?;

        let rtn = match method {
            WorktopMethod::Put => {
                let input: WorktopPutInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let bucket = system_api
                    .node_drop(RENodeId::Bucket(input.bucket.0))?
                    .into();
                let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
                let mut raw_mut = substate_mut.get_raw_mut();
                let worktop = raw_mut.worktop();
                worktop
                    .put(bucket)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?;
                substate_mut.flush()?;

                Ok(ScryptoValue::from_typed(&()))
            }
            WorktopMethod::TakeAmount => {
                let input: WorktopTakeAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;

                let maybe_resource = {
                    let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let worktop = raw_mut.worktop();
                    let maybe_resource = worktop
                        .take(input.amount, input.resource_address)
                        .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?;
                    substate_mut.flush()?;
                    maybe_resource
                };

                let resource_resource = if let Some(resource) = maybe_resource {
                    resource
                } else {
                    let resource_type = {
                        let resource_id =
                            RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                        let offset =
                            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                        let resource_handle =
                            system_api.lock_substate(resource_id, offset, LockFlags::empty())?;
                        let substate_ref = system_api.get_ref(resource_handle)?;
                        substate_ref.resource_manager().resource_type
                    };

                    Resource::new_empty(input.resource_address, resource_type)
                };
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(BucketSubstate::new(resource_resource)))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopMethod::TakeAll => {
                let input: WorktopTakeAllInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;

                let maybe_resource = {
                    let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let worktop = raw_mut.worktop();
                    let maybe_resource = worktop
                        .take_all(input.resource_address)
                        .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?;
                    substate_mut.flush()?;
                    maybe_resource
                };

                let resource_resource = if let Some(resource) = maybe_resource {
                    resource
                } else {
                    let resource_type = {
                        let resource_id =
                            RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                        let offset =
                            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                        let resource_handle =
                            system_api.lock_substate(resource_id, offset, LockFlags::empty())?;
                        let substate_ref = system_api.get_ref(resource_handle)?;
                        substate_ref.resource_manager().resource_type
                    };

                    Resource::new_empty(input.resource_address, resource_type)
                };

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(BucketSubstate::new(resource_resource)))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopMethod::TakeNonFungibles => {
                let input: WorktopTakeNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let maybe_resource = {
                    let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let worktop = raw_mut.worktop();
                    let maybe_resource = worktop
                        .take_non_fungibles(&input.ids, input.resource_address)
                        .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?;
                    substate_mut.flush()?;
                    maybe_resource
                };

                let resource_resource = if let Some(resource) = maybe_resource {
                    resource
                } else {
                    let resource_type = {
                        let resource_id =
                            RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                        let offset =
                            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                        let resource_handle =
                            system_api.lock_substate(resource_id, offset, LockFlags::empty())?;
                        let substate_ref = system_api.get_ref(resource_handle)?;
                        substate_ref.resource_manager().resource_type
                    };

                    Resource::new_empty(input.resource_address, resource_type)
                };

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(BucketSubstate::new(resource_resource)))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopMethod::AssertContains => {
                let input: WorktopAssertContainsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let substate_ref = system_api.get_ref(worktop_handle)?;
                let worktop = substate_ref.worktop();
                if worktop.total_amount(input.resource_address).is_zero() {
                    Err(InvokeError::Error(WorktopError::AssertionFailed))
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            WorktopMethod::AssertContainsAmount => {
                let input: WorktopAssertContainsAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let substate_ref = system_api.get_ref(worktop_handle)?;
                let worktop = substate_ref.worktop();
                if worktop.total_amount(input.resource_address) < input.amount {
                    Err(InvokeError::Error(WorktopError::AssertionFailed))
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            WorktopMethod::AssertContainsNonFungibles => {
                let input: WorktopAssertContainsNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let substate_ref = system_api.get_ref(worktop_handle)?;
                let worktop = substate_ref.worktop();
                if !worktop
                    .total_ids(input.resource_address)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?
                    .is_superset(&input.ids)
                {
                    Err(InvokeError::Error(WorktopError::AssertionFailed))
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            WorktopMethod::Drain => {
                let _: WorktopDrainInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let mut resources = Vec::new();
                {
                    let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let worktop = raw_mut.worktop();
                    for (_, resource) in worktop.resources.drain() {
                        let taken = resource.borrow_mut().take_all_liquid().map_err(|e| {
                            InvokeError::Error(WorktopError::ResourceOperationError(e))
                        })?;
                        if !taken.is_empty() {
                            resources.push(taken);
                        }
                    }
                }

                let mut buckets = Vec::new();
                for resource in resources {
                    let bucket_id = system_api
                        .node_create(HeapRENode::Bucket(BucketSubstate::new(resource)))?
                        .into();
                    buckets.push(scrypto::resource::Bucket(bucket_id))
                }
                Ok(ScryptoValue::from_typed(&buckets))
            }
        }?;

        Ok(rtn)
    }
}
