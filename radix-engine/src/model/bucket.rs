use sbor::rust::cell::{Ref, RefCell, RefMut};
use sbor::rust::collections::BTreeSet;
use sbor::rust::collections::HashMap;
use sbor::rust::rc::Rc;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::types::*;
use scrypto::prelude::{
    BucketCreateProofInput, BucketGetAmountInput, BucketGetNonFungibleIdsInput,
    BucketGetResourceAddressInput, BucketPutInput, BucketTakeInput, BucketTakeNonFungiblesInput,
    ConsumingBucketBurnInput,
};
use scrypto::values::ScryptoValue;

use crate::engine::{SubstateId, SystemApi};
use crate::fee::CostUnitCounter;
use crate::fee::CostUnitCounterError;
use crate::model::{
    Proof, ProofError, ResourceContainer, ResourceContainerError, ResourceContainerId,
};
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq)]
pub enum BucketError {
    InvalidDivisibility,
    InvalidRequestData(DecodeError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ResourceContainerError(ResourceContainerError),
    ProofError(ProofError),
    CouldNotCreateProof,
    MethodNotFound(String),
    CostingError(CostUnitCounterError),
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
            ResourceType::NonFungible => {
                self.create_proof_by_ids(&self.total_ids().unwrap(), container_id)
            }
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

    fn total_amount(&self) -> Decimal {
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

    pub fn main<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: CostUnitCounter,
    >(
        bucket_id: BucketId,
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, BucketError> {
        let node_id = RENodeId::Bucket(bucket_id);
        let mut node_ref = system_api
            .borrow_node_mut(&node_id)
            .map_err(BucketError::CostingError)?;
        let bucket0 = node_ref.bucket();

        let rtn = match method_name {
            "take" => {
                let input: BucketTakeInput =
                    scrypto_decode(&arg.raw).map_err(|e| BucketError::InvalidRequestData(e))?;
                let container = bucket0
                    .take(input.amount)
                    .map_err(BucketError::ResourceContainerError)?;
                let bucket_id = system_api
                    .create_node(Bucket::new(container))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "take_non_fungibles" => {
                let input: BucketTakeNonFungiblesInput =
                    scrypto_decode(&arg.raw).map_err(|e| BucketError::InvalidRequestData(e))?;
                let container = bucket0
                    .take_non_fungibles(&input.ids)
                    .map_err(BucketError::ResourceContainerError)?;
                let bucket_id = system_api
                    .create_node(Bucket::new(container))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "non_fungible_ids" => {
                let _: BucketGetNonFungibleIdsInput =
                    scrypto_decode(&arg.raw).map_err(|e| BucketError::InvalidRequestData(e))?;
                let ids = bucket0
                    .total_ids()
                    .map_err(BucketError::ResourceContainerError)?;
                Ok(ScryptoValue::from_typed(&ids))
            }
            "put" => {
                let input: BucketPutInput =
                    scrypto_decode(&arg.raw).map_err(|e| BucketError::InvalidRequestData(e))?;
                let other_bucket = system_api
                    .drop_node(&RENodeId::Bucket(input.bucket.0))
                    .map_err(BucketError::CostingError)?
                    .into();
                bucket0
                    .put(other_bucket)
                    .map_err(BucketError::ResourceContainerError)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            "amount" => {
                let _: BucketGetAmountInput =
                    scrypto_decode(&arg.raw).map_err(|e| BucketError::InvalidRequestData(e))?;
                Ok(ScryptoValue::from_typed(&bucket0.total_amount()))
            }
            "resource_address" => {
                let _: BucketGetResourceAddressInput =
                    scrypto_decode(&arg.raw).map_err(|e| BucketError::InvalidRequestData(e))?;
                Ok(ScryptoValue::from_typed(&bucket0.resource_address()))
            }
            "create_proof" => {
                let _: BucketCreateProofInput =
                    scrypto_decode(&arg.raw).map_err(|e| BucketError::InvalidRequestData(e))?;
                let proof = bucket0
                    .create_proof(bucket_id)
                    .map_err(BucketError::ProofError)?;
                let proof_id = system_api.create_node(proof).unwrap().into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            _ => Err(BucketError::MethodNotFound(method_name.to_string())),
        }?;

        system_api
            .return_node_mut(node_ref)
            .map_err(BucketError::CostingError)?;

        Ok(rtn)
    }

    pub fn consuming_main<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: CostUnitCounter,
    >(
        node_id: RENodeId,
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, BucketError> {
        match method_name {
            "burn" => {
                let _: ConsumingBucketBurnInput =
                    scrypto_decode(&arg.raw).map_err(|e| BucketError::InvalidRequestData(e))?;

                let bucket: Bucket = system_api
                    .drop_node(&node_id)
                    .map_err(BucketError::CostingError)?
                    .into();

                // Notify resource manager, TODO: Should not need to notify manually
                let resource_address = bucket.resource_address();
                let resource_id = RENodeId::Resource(resource_address);
                let mut value = system_api
                    .borrow_node_mut(&resource_id)
                    .map_err(BucketError::CostingError)?;
                let resource_manager = value.resource_manager();
                resource_manager.burn(bucket.total_amount());
                if matches!(resource_manager.resource_type(), ResourceType::NonFungible) {
                    for id in bucket.total_ids().unwrap() {
                        let address = SubstateId::NonFungible(resource_address, id);
                        system_api.take_substate(address).expect("Should not fail.");
                    }
                }
                system_api
                    .return_node_mut(value)
                    .map_err(BucketError::CostingError)?;

                Ok(ScryptoValue::from_typed(&()))
            }
            _ => Err(BucketError::MethodNotFound(method_name.to_string())),
        }
    }
}
