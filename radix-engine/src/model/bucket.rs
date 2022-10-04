use crate::engine::{HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{
    InvokeError, Proof, ProofError, ResourceContainer, ResourceContainerError, ResourceContainerId,
};
use crate::types::*;
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum BucketError {
    InvalidDivisibility,
    InvalidRequestData(DecodeError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ResourceContainerError(ResourceContainerError),
    ProofError(ProofError),
    CouldNotCreateProof,
    MethodNotFound(BucketFnIdentifier),
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

    fn put(&mut self, other: Bucket) -> Result<(), ResourceContainerError> {
        self.borrow_container_mut().put(other.into_container()?)
    }

    fn take(&mut self, amount: Decimal) -> Result<ResourceContainer, ResourceContainerError> {
        self.borrow_container_mut().take_by_amount(amount)
    }

    fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<ResourceContainer, ResourceContainerError> {
        self.borrow_container_mut().take_by_ids(ids)
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
            .borrow_container_mut()
            .lock_by_amount(amount)
            .map_err(ProofError::ResourceContainerError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.container.clone(), locked_amount_or_ids.clone()),
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
            .borrow_container_mut()
            .lock_by_ids(ids)
            .map_err(ProofError::ResourceContainerError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.container.clone(), locked_amount_or_ids.clone()),
        );
        Proof::new(
            self.resource_address(),
            self.resource_type(),
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.borrow_container().resource_address()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.borrow_container().resource_type()
    }

    pub fn total_amount(&self) -> Decimal {
        self.borrow_container().total_amount()
    }

    fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ResourceContainerError> {
        self.borrow_container().total_ids()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_container().is_locked()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_container().is_empty()
    }

    pub fn into_container(self) -> Result<ResourceContainer, ResourceContainerError> {
        Rc::try_unwrap(self.container)
            .map_err(|_| ResourceContainerError::ContainerLocked)
            .map(|c| c.into_inner())
    }

    fn borrow_container(&self) -> Ref<ResourceContainer> {
        self.container.borrow()
    }

    fn borrow_container_mut(&mut self) -> RefMut<ResourceContainer> {
        self.container.borrow_mut()
    }

    pub fn main<'s, Y, W, I, R>(
        bucket_id: BucketId,
        bucket_fn: BucketFnIdentifier,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let substate_id = SubstateId::Bucket(bucket_id);
        let mut node_ref = system_api
            .substate_borrow_mut(&substate_id)
            .map_err(InvokeError::Downstream)?;
        let bucket0 = node_ref.bucket();

        let rtn = match bucket_fn {
            BucketFnIdentifier::Take => {
                let input: BucketTakeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let container = bucket0
                    .take(input.amount)
                    .map_err(|e| InvokeError::Error(BucketError::ResourceContainerError(e)))?;
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            BucketFnIdentifier::TakeNonFungibles => {
                let input: BucketTakeNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let container = bucket0
                    .take_non_fungibles(&input.ids)
                    .map_err(|e| InvokeError::Error(BucketError::ResourceContainerError(e)))?;
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            BucketFnIdentifier::GetNonFungibleIds => {
                let _: BucketGetNonFungibleIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let ids = bucket0
                    .total_ids()
                    .map_err(|e| InvokeError::Error(BucketError::ResourceContainerError(e)))?;
                Ok(ScryptoValue::from_typed(&ids))
            }
            BucketFnIdentifier::Put => {
                let input: BucketPutInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let other_bucket = system_api
                    .node_drop(&RENodeId::Bucket(input.bucket.0))
                    .map_err(InvokeError::Downstream)?
                    .into();
                bucket0
                    .put(other_bucket)
                    .map_err(|e| InvokeError::Error(BucketError::ResourceContainerError(e)))?;
                Ok(ScryptoValue::from_typed(&()))
            }
            BucketFnIdentifier::GetAmount => {
                let _: BucketGetAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                Ok(ScryptoValue::from_typed(&bucket0.total_amount()))
            }
            BucketFnIdentifier::GetResourceAddress => {
                let _: BucketGetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                Ok(ScryptoValue::from_typed(&bucket0.resource_address()))
            }
            BucketFnIdentifier::CreateProof => {
                let _: BucketCreateProofInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let proof = bucket0
                    .create_proof(bucket_id)
                    .map_err(|e| InvokeError::Error(BucketError::ProofError(e)))?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            _ => Err(InvokeError::Error(BucketError::MethodNotFound(bucket_fn))),
        }?;

        system_api
            .substate_return_mut(node_ref)
            .map_err(InvokeError::Downstream)?;

        Ok(rtn)
    }

    pub fn consuming_main<'s, Y, W, I, R>(
        node_id: RENodeId,
        bucket_fn: BucketFnIdentifier,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match bucket_fn {
            BucketFnIdentifier::Burn => {
                let _: ConsumingBucketBurnInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;

                let bucket: Bucket = system_api
                    .node_drop(&node_id)
                    .map_err(InvokeError::Downstream)?
                    .into();

                // Notify resource manager, TODO: Should not need to notify manually
                let resource_address = bucket.resource_address();
                let resource_substate_id = SubstateId::ResourceManager(resource_address);
                let mut value = system_api
                    .substate_borrow_mut(&resource_substate_id)
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = value.resource_manager();
                resource_manager.burn(bucket.total_amount());
                if matches!(resource_manager.resource_type(), ResourceType::NonFungible) {
                    for id in bucket
                        .total_ids()
                        .expect("Failed to list non-fungible IDs on non-fungible Bucket")
                    {
                        let address = SubstateId::NonFungible(resource_address, id);
                        system_api
                            .substate_take(address)
                            .map_err(InvokeError::Downstream)?;
                    }
                }
                system_api
                    .substate_return_mut(value)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&()))
            }
            _ => Err(InvokeError::Error(BucketError::MethodNotFound(bucket_fn))),
        }
    }
}
