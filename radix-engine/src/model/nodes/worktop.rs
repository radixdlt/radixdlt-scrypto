use crate::engine::{DropFailure, HeapRENode, InvokeError, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{Bucket, LockableResource, Resource, ResourceOperationError};
use crate::types::*;
use crate::wasm::*;
use scrypto::core::{FnIdent, MethodIdent, ReceiverMethodIdent};

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
    resources: HashMap<ResourceAddress, Rc<RefCell<LockableResource>>>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum WorktopError {
    InvalidRequestData(DecodeError),
    MethodNotFound(String),
    ResourceOperationError(ResourceOperationError),
    ResourceNotFound(ResourceAddress),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    AssertionFailed,
}

impl Worktop {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn drop(self) -> Result<(), DropFailure> {
        for (_address, container) in self.resources {
            if !container.borrow().is_empty() {
                return Err(DropFailure::Worktop);
            }
        }

        Ok(())
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), ResourceOperationError> {
        let resource_address = other.resource_address();
        let resource = other.resource()?;
        if let Some(mut container) = self.borrow_resource_mut(resource_address) {
            return container.put(resource);
        }
        self.put_container(resource_address, resource.into());
        Ok(())
    }

    fn take(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> Result<Option<Resource>, ResourceOperationError> {
        if let Some(mut container) = self.borrow_resource_mut(resource_address) {
            container.take_by_amount(amount).map(Option::Some)
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
        if let Some(mut container) = self.borrow_resource_mut(resource_address) {
            container.take_by_ids(ids).map(Option::Some)
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
        if let Some(mut container) = self.borrow_resource_mut(resource_address) {
            Ok(Some(container.take_all_liquid()?))
        } else {
            Ok(None)
        }
    }

    pub fn resource_addresses(&self) -> Vec<ResourceAddress> {
        self.resources.keys().cloned().collect()
    }

    pub fn total_amount(&self, resource_address: ResourceAddress) -> Decimal {
        if let Some(container) = self.borrow_resource(resource_address) {
            container.total_amount()
        } else {
            Decimal::zero()
        }
    }

    pub fn total_ids(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<BTreeSet<NonFungibleId>, ResourceOperationError> {
        if let Some(container) = self.borrow_resource(resource_address) {
            container.total_ids()
        } else {
            Ok(BTreeSet::new())
        }
    }

    pub fn is_locked(&self) -> bool {
        for resource_address in self.resource_addresses() {
            if let Some(container) = self.borrow_resource(resource_address) {
                if container.is_locked() {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        for resource_address in self.resource_addresses() {
            if let Some(container) = self.borrow_resource(resource_address) {
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

    // Note that this method overwrites existing container if any
    fn put_container(&mut self, resource_address: ResourceAddress, container: LockableResource) {
        self.resources
            .insert(resource_address, Rc::new(RefCell::new(container)));
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
        let rtn = match method {
            WorktopMethod::Put => {
                let input: WorktopPutInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let bucket = system_api
                    .node_drop(&RENodeId::Bucket(input.bucket.0))
                    .map_err(|e| InvokeError::Downstream(e))?
                    .into();
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Worktop)
                    .map_err(InvokeError::downstream)?;
                let worktop = node_ref.worktop_mut();
                worktop
                    .put(bucket)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?;
                Ok(ScryptoValue::from_typed(&()))
            }
            WorktopMethod::TakeAmount => {
                let input: WorktopTakeAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Worktop)
                    .map_err(InvokeError::downstream)?;
                let worktop = node_ref.worktop_mut();
                let maybe_container = worktop
                    .take(input.amount, input.resource_address)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    // TODO: substate read instead of invoke?
                    let resource_type = {
                        let result = system_api
                            .invoke(
                                FnIdent::Method(ReceiverMethodIdent {
                                    receiver: Receiver::Ref(RENodeId::Global(
                                        GlobalAddress::Resource(input.resource_address),
                                    )),
                                    method_ident: MethodIdent::Native(
                                        NativeMethod::ResourceManager(
                                            ResourceManagerMethod::GetResourceType,
                                        ),
                                    ),
                                }),
                                ScryptoValue::from_typed(&ResourceManagerGetResourceTypeInput {}),
                            )
                            .map_err(InvokeError::Downstream)?;
                        let resource_type: ResourceType = scrypto_decode(&result.raw).unwrap();
                        resource_type
                    };

                    Resource::new_empty(input.resource_address, resource_type)
                };
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(resource_container)))
                    .map_err(|e| InvokeError::Downstream(e))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopMethod::TakeAll => {
                let input: WorktopTakeAllInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Worktop)
                    .map_err(InvokeError::downstream)?;
                let worktop = node_ref.worktop_mut();
                let maybe_container = worktop
                    .take_all(input.resource_address)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    // TODO: substate read instead of invoke?
                    let resource_type = {
                        let result = system_api
                            .invoke(
                                FnIdent::Method(ReceiverMethodIdent {
                                    receiver: Receiver::Ref(RENodeId::Global(
                                        GlobalAddress::Resource(input.resource_address),
                                    )),
                                    method_ident: MethodIdent::Native(
                                        NativeMethod::ResourceManager(
                                            ResourceManagerMethod::GetResourceType,
                                        ),
                                    ),
                                }),
                                ScryptoValue::from_typed(&ResourceManagerGetResourceTypeInput {}),
                            )
                            .map_err(InvokeError::Downstream)?;
                        let resource_type: ResourceType = scrypto_decode(&result.raw).unwrap();
                        resource_type
                    };

                    Resource::new_empty(input.resource_address, resource_type)
                };

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(resource_container)))
                    .map_err(|e| InvokeError::Downstream(e))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopMethod::TakeNonFungibles => {
                let input: WorktopTakeNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Worktop)
                    .map_err(InvokeError::downstream)?;
                let worktop = node_ref.worktop_mut();
                let maybe_container = worktop
                    .take_non_fungibles(&input.ids, input.resource_address)
                    .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?;
                let resource_container = if let Some(container) = maybe_container {
                    container
                } else {
                    // TODO: substate read instead of invoke?
                    let resource_type = {
                        let result = system_api
                            .invoke(
                                FnIdent::Method(ReceiverMethodIdent {
                                    receiver: Receiver::Ref(RENodeId::Global(
                                        GlobalAddress::Resource(input.resource_address),
                                    )),
                                    method_ident: MethodIdent::Native(
                                        NativeMethod::ResourceManager(
                                            ResourceManagerMethod::GetResourceType,
                                        ),
                                    ),
                                }),
                                ScryptoValue::from_typed(&ResourceManagerGetResourceTypeInput {}),
                            )
                            .map_err(InvokeError::Downstream)?;
                        let resource_type: ResourceType = scrypto_decode(&result.raw).unwrap();
                        resource_type
                    };

                    Resource::new_empty(input.resource_address, resource_type)
                };

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(resource_container)))
                    .map_err(|e| InvokeError::Downstream(e))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            WorktopMethod::AssertContains => {
                let input: WorktopAssertContainsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Worktop)
                    .map_err(InvokeError::downstream)?;
                let worktop = node_ref.worktop_mut();
                if worktop.total_amount(input.resource_address).is_zero() {
                    Err(InvokeError::Error(WorktopError::AssertionFailed))
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            WorktopMethod::AssertContainsAmount => {
                let input: WorktopAssertContainsAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Worktop)
                    .map_err(InvokeError::downstream)?;
                let worktop = node_ref.worktop_mut();
                if worktop.total_amount(input.resource_address) < input.amount {
                    Err(InvokeError::Error(WorktopError::AssertionFailed))
                } else {
                    Ok(ScryptoValue::from_typed(&()))
                }
            }
            WorktopMethod::AssertContainsNonFungibles => {
                let input: WorktopAssertContainsNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(WorktopError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Worktop)
                    .map_err(InvokeError::downstream)?;
                let worktop = node_ref.worktop_mut();
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
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Worktop)
                    .map_err(InvokeError::downstream)?;
                let worktop = node_ref.worktop_mut();
                let mut resources = Vec::new();
                for (_, resource) in worktop.resources.drain() {
                    let taken = resource
                        .borrow_mut()
                        .take_all_liquid()
                        .map_err(|e| InvokeError::Error(WorktopError::ResourceOperationError(e)))?;
                    if !taken.is_empty() {
                        resources.push(taken);
                    }
                }
                let mut buckets = Vec::new();
                for resource in resources {
                    let bucket_id = system_api
                        .node_create(HeapRENode::Bucket(Bucket::new(resource)))
                        .map_err(|e| InvokeError::Downstream(e))?
                        .into();
                    buckets.push(scrypto::resource::Bucket(bucket_id))
                }
                Ok(ScryptoValue::from_typed(&buckets))
            }
        }?;

        Ok(rtn)
    }
}
