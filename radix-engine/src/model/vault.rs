use sbor::rust::cell::{Ref, RefCell, RefMut};
use sbor::rust::collections::BTreeSet;
use sbor::rust::collections::HashMap;
use sbor::rust::rc::Rc;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::types::*;
use scrypto::prelude::{
    VaultCreateProofByIdsInput, VaultCreateProofInput, VaultGetAmountInput,
    VaultGetNonFungibleIdsInput, VaultPutInput, VaultTakeInput,
};
use scrypto::resource::{
    VaultCreateProofByAmountInput, VaultGetResourceAddressInput, VaultTakeNonFungiblesInput,
};
use scrypto::values::ScryptoValue;

use crate::engine::SystemApi;
use crate::fee::CostUnitCounterError;
use crate::ledger::ReadableSubstateStore;
use crate::model::VaultError::MethodNotFound;
use crate::model::{
    Bucket, Proof, ProofError, ResourceContainer, ResourceContainerError, ResourceContainerId,
};
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq)]
pub enum VaultError {
    InvalidRequestData(DecodeError),
    ResourceContainerError(ResourceContainerError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ProofError(ProofError),
    CouldNotCreateProof,
    MethodNotFound,
    CostingError(CostUnitCounterError),
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

    pub fn put(&mut self, other: Bucket) -> Result<(), ResourceContainerError> {
        self.borrow_container_mut().put(other.into_container()?)
    }

    fn take(&mut self, amount: Decimal) -> Result<ResourceContainer, VaultError> {
        let container = self
            .borrow_container_mut()
            .take_by_amount(amount)
            .map_err(VaultError::ResourceContainerError)?;
        Ok(container)
    }

    fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<ResourceContainer, VaultError> {
        let container = self
            .borrow_container_mut()
            .take_by_ids(ids)
            .map_err(VaultError::ResourceContainerError)?;
        Ok(container)
    }

    pub fn create_proof(&mut self, container_id: ResourceContainerId) -> Result<Proof, ProofError> {
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

    pub fn total_amount(&self) -> Decimal {
        self.borrow_container().total_amount()
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ResourceContainerError> {
        self.borrow_container().total_ids()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_container().is_locked()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_container().is_empty()
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
        Y: SystemApi<'p, 's, W, I, S>,
        W: WasmEngine<I>,
        I: WasmInstance,
        S: ReadableSubstateStore,
    >(
        vault_id: VaultId,
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, VaultError> {
        let value_id = ValueId::Vault(vault_id.clone());
        let mut ref_mut = system_api
            .borrow_value_mut(&value_id)
            .map_err(VaultError::CostingError)?;
        let vault = ref_mut.vault();

        let rtn = match method_name {
            "put" => {
                let input: VaultPutInput =
                    scrypto_decode(&arg.raw).map_err(|e| VaultError::InvalidRequestData(e))?;
                let bucket = system_api
                    .drop_value(&ValueId::Bucket(input.bucket.0))
                    .map_err(VaultError::CostingError)?
                    .into();
                vault
                    .put(bucket)
                    .map_err(VaultError::ResourceContainerError)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            "take" => {
                let input: VaultTakeInput =
                    scrypto_decode(&arg.raw).map_err(|e| VaultError::InvalidRequestData(e))?;
                let container = vault.take(input.amount)?;
                let bucket_id = system_api
                    .create_value(Bucket::new(container))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "take_non_fungibles" => {
                let input: VaultTakeNonFungiblesInput =
                    scrypto_decode(&arg.raw).map_err(|e| VaultError::InvalidRequestData(e))?;
                let container = vault.take_non_fungibles(&input.non_fungible_ids)?;
                let bucket_id = system_api
                    .create_value(Bucket::new(container))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "amount" => {
                let _: VaultGetAmountInput =
                    scrypto_decode(&arg.raw).map_err(|e| VaultError::InvalidRequestData(e))?;
                let amount = vault.total_amount();
                Ok(ScryptoValue::from_typed(&amount))
            }
            "resource_address" => {
                let _: VaultGetResourceAddressInput =
                    scrypto_decode(&arg.raw).map_err(|e| VaultError::InvalidRequestData(e))?;
                let resource_address = vault.resource_address();
                Ok(ScryptoValue::from_typed(&resource_address))
            }
            "non_fungible_ids" => {
                let _: VaultGetNonFungibleIdsInput =
                    scrypto_decode(&arg.raw).map_err(|e| VaultError::InvalidRequestData(e))?;
                let ids = vault
                    .total_ids()
                    .map_err(VaultError::ResourceContainerError)?;
                Ok(ScryptoValue::from_typed(&ids))
            }
            "create_proof" => {
                let _: VaultCreateProofInput =
                    scrypto_decode(&arg.raw).map_err(|e| VaultError::InvalidRequestData(e))?;
                let proof = vault
                    .create_proof(ResourceContainerId::Vault(vault_id))
                    .map_err(VaultError::ProofError)?;
                let proof_id = system_api.create_value(proof).unwrap().into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            "create_proof_by_amount" => {
                let input: VaultCreateProofByAmountInput =
                    scrypto_decode(&arg.raw).map_err(|e| VaultError::InvalidRequestData(e))?;
                let proof = vault
                    .create_proof_by_amount(input.amount, ResourceContainerId::Vault(vault_id))
                    .map_err(VaultError::ProofError)?;
                let proof_id = system_api.create_value(proof).unwrap().into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            "create_proof_by_ids" => {
                let input: VaultCreateProofByIdsInput =
                    scrypto_decode(&arg.raw).map_err(|e| VaultError::InvalidRequestData(e))?;
                let proof = vault
                    .create_proof_by_ids(&input.ids, ResourceContainerId::Vault(vault_id))
                    .map_err(VaultError::ProofError)?;
                let proof_id = system_api.create_value(proof).unwrap().into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            _ => Err(MethodNotFound),
        }?;

        system_api
            .return_value_mut(ref_mut)
            .map_err(VaultError::CostingError)?;

        Ok(rtn)
    }
}
