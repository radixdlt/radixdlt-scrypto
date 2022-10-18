use crate::engine::{HeapRENode, InvokeError, LockFlags, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{BucketSubstate, Resource, ResourceOperationError};
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

pub struct Worktop;

impl Worktop {
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
                        let resource_handle = system_api.lock_substate(
                            resource_id,
                            offset,
                            LockFlags::read_only(),
                        )?;
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
                        let resource_handle = system_api.lock_substate(
                            resource_id,
                            offset,
                            LockFlags::read_only(),
                        )?;
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
                        let resource_handle = system_api.lock_substate(
                            resource_id,
                            offset,
                            LockFlags::read_only(),
                        )?;
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
