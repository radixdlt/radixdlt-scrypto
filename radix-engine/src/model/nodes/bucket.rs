use scrypto::core::{FnIdent, MethodIdent, ReceiverMethodIdent};
use scrypto::resource::ResourceManagerBurnInput;

use crate::engine::{HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{
    InvokeError, LockableResource, Proof, ProofError, Resource, ResourceContainerId,
    ResourceOperationError,
};
use crate::types::*;
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum BucketError {
    InvalidDivisibility,
    InvalidRequestData(DecodeError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ResourceOperationError(ResourceOperationError),
    ProofError(ProofError),
    CouldNotCreateProof,
    MethodNotFound(BucketMethod),
}

/// A transient resource container.
#[derive(Debug)]
pub struct Bucket {
    resource: Rc<RefCell<LockableResource>>,
}

impl Bucket {
    pub fn new(resource: Resource) -> Self {
        Self {
            resource: Rc::new(RefCell::new(resource.into())),
        }
    }

    fn put(&mut self, other: Bucket) -> Result<(), ResourceOperationError> {
        self.borrow_resource_mut().put(other.resource()?)
    }

    fn take(&mut self, amount: Decimal) -> Result<Resource, ResourceOperationError> {
        self.borrow_resource_mut().take_by_amount(amount)
    }

    fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<Resource, ResourceOperationError> {
        self.borrow_resource_mut().take_by_ids(ids)
    }

    pub fn create_proof(&mut self, self_bucket_id: BucketId) -> Result<Proof, ProofError> {
        let container_id = ResourceContainerId::Bucket(self_bucket_id);
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), container_id)
            }
            ResourceType::NonFungible => self.create_proof_by_ids(
                &self
                    .total_ids()
                    .expect("Failed to list non-fungible IDs on non-fungible Bucket"),
                container_id,
            ),
        }
    }

    pub fn create_proof_by_amount(
        &mut self,
        amount: Decimal,
        container_id: ResourceContainerId,
    ) -> Result<Proof, ProofError> {
        // lock the specified amount
        let locked_amount_or_ids = self
            .borrow_resource_mut()
            .lock_by_amount(amount)
            .map_err(ProofError::ResourceOperationError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.resource.clone(), locked_amount_or_ids.clone()),
        );
        Proof::new(
            self.resource_address(),
            self.resource_type(),
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn create_proof_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        container_id: ResourceContainerId,
    ) -> Result<Proof, ProofError> {
        // lock the specified id set
        let locked_amount_or_ids = self
            .borrow_resource_mut()
            .lock_by_ids(ids)
            .map_err(ProofError::ResourceOperationError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.resource.clone(), locked_amount_or_ids.clone()),
        );
        Proof::new(
            self.resource_address(),
            self.resource_type(),
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.borrow_resource().resource_address()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.borrow_resource().resource_type()
    }

    pub fn total_amount(&self) -> Decimal {
        self.borrow_resource().total_amount()
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ResourceOperationError> {
        self.borrow_resource().total_ids()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_resource().is_locked()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_resource().is_empty()
    }

    pub fn resource(self) -> Result<Resource, ResourceOperationError> {
        Rc::try_unwrap(self.resource)
            .map_err(|_| ResourceOperationError::ResourceLocked)
            .map(|c| c.into_inner())
            .map(Into::into)
    }

    fn borrow_resource(&self) -> Ref<LockableResource> {
        self.resource.borrow()
    }

    fn borrow_resource_mut(&mut self) -> RefMut<LockableResource> {
        self.resource.borrow_mut()
    }

    pub fn main<'s, Y, W, I, R>(
        bucket_id: BucketId,
        method: BucketMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let rtn = match method {
            BucketMethod::Take => {
                let input: BucketTakeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let mut node_ref = system_api.borrow_node_mut(&RENodeId::Bucket(bucket_id))?;
                let bucket0 = node_ref.bucket_mut();

                let container = bucket0
                    .take(input.amount)
                    .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            BucketMethod::TakeNonFungibles => {
                let input: BucketTakeNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let mut node_ref = system_api.borrow_node_mut(&RENodeId::Bucket(bucket_id))?;
                let bucket0 = node_ref.bucket_mut();

                let container = bucket0
                    .take_non_fungibles(&input.ids)
                    .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            BucketMethod::GetNonFungibleIds => {
                let _: BucketGetNonFungibleIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let mut node_ref = system_api.borrow_node_mut(&RENodeId::Bucket(bucket_id))?;
                let bucket0 = node_ref.bucket_mut();

                let ids = bucket0
                    .total_ids()
                    .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
                Ok(ScryptoValue::from_typed(&ids))
            }
            BucketMethod::Put => {
                let input: BucketPutInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let other_bucket = system_api
                    .node_drop(RENodeId::Bucket(input.bucket.0))?
                    .into();
                let mut node_ref = system_api.borrow_node_mut(&RENodeId::Bucket(bucket_id))?;
                let bucket0 = node_ref.bucket_mut();

                bucket0
                    .put(other_bucket)
                    .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
                Ok(ScryptoValue::from_typed(&()))
            }
            BucketMethod::GetAmount => {
                let _: BucketGetAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let mut node_ref = system_api.borrow_node_mut(&RENodeId::Bucket(bucket_id))?;
                let bucket0 = node_ref.bucket_mut();

                Ok(ScryptoValue::from_typed(&bucket0.total_amount()))
            }
            BucketMethod::GetResourceAddress => {
                let _: BucketGetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let mut node_ref = system_api.borrow_node_mut(&RENodeId::Bucket(bucket_id))?;
                let bucket0 = node_ref.bucket_mut();

                Ok(ScryptoValue::from_typed(&bucket0.resource_address()))
            }
            BucketMethod::CreateProof => {
                let _: BucketCreateProofInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let mut node_ref = system_api.borrow_node_mut(&RENodeId::Bucket(bucket_id))?;
                let bucket0 = node_ref.bucket_mut();
                let proof = bucket0
                    .create_proof(bucket_id)
                    .map_err(|e| InvokeError::Error(BucketError::ProofError(e)))?;
                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            _ => Err(InvokeError::Error(BucketError::MethodNotFound(method))),
        }?;

        Ok(rtn)
    }

    pub fn consuming_main<'s, Y, W, I, R>(
        node_id: RENodeId,
        method: BucketMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match method {
            BucketMethod::Burn => {
                let _: ConsumingBucketBurnInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;

                let node_ref = system_api.borrow_node(&node_id)?;
                let bucket = node_ref.bucket();
                let resource_address = bucket.resource_address();

                let bucket_id = match node_id {
                    RENodeId::Bucket(bucket_id) => bucket_id,
                    _ => panic!("Unexpected"),
                };

                system_api
                    .invoke(
                        FnIdent::Method(ReceiverMethodIdent {
                            receiver: Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                            method_ident: MethodIdent::Native(NativeMethod::ResourceManager(
                                ResourceManagerMethod::Burn,
                            )),
                        }),
                        ScryptoValue::from_typed(&ResourceManagerBurnInput {
                            bucket: scrypto::resource::Bucket(bucket_id),
                        }),
                    )
                    .map_err(InvokeError::Downstream)
            }
            _ => Err(InvokeError::Error(BucketError::MethodNotFound(method))),
        }
    }
}
