use crate::engine::{HeapRENode, SystemApi};
use crate::fee::{FeeReserve, FeeReserveError};
use crate::model::{
    Bucket, InvokeError, LockableResource, Proof, ProofError, Resource, ResourceContainerId,
    ResourceOperationError,
};
use crate::types::*;
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum VaultError {
    InvalidRequestData(DecodeError),
    ResourceOperationError(ResourceOperationError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ProofError(ProofError),
    CouldNotCreateProof,
    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
    LockFeeRepayFailure(FeeReserveError),
}

/// A persistent resource container.
#[derive(Debug)]
pub struct Vault {
    resource: Rc<RefCell<LockableResource>>,
}

impl Vault {
    pub fn new(resource: Resource) -> Self {
        Self {
            resource: Rc::new(RefCell::new(resource.into())),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), ResourceOperationError> {
        self.borrow_resource_mut().put(other.resource()?)
    }

    fn take(&mut self, amount: Decimal) -> Result<Resource, InvokeError<VaultError>> {
        let resource = self
            .borrow_resource_mut()
            .take_by_amount(amount)
            .map_err(|e| InvokeError::Error(VaultError::ResourceOperationError(e)))?;
        Ok(resource)
    }

    fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<Resource, InvokeError<VaultError>> {
        let resource = self
            .borrow_resource_mut()
            .take_by_ids(ids)
            .map_err(|e| InvokeError::Error(VaultError::ResourceOperationError(e)))?;
        Ok(resource)
    }

    pub fn create_proof(&mut self, container_id: ResourceContainerId) -> Result<Proof, ProofError> {
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), container_id)
            }
            ResourceType::NonFungible => self.create_proof_by_ids(
                &self
                    .total_ids()
                    .expect("Failed to list non-fungible IDs of non-fungible vault"),
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

    pub fn borrow_resource(&self) -> Ref<LockableResource> {
        self.resource.borrow()
    }

    pub fn borrow_resource_mut(&mut self) -> RefMut<LockableResource> {
        self.resource.borrow_mut()
    }

    pub fn main<'s, Y, W, I, R>(
        vault_id: VaultId,
        method: VaultMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<VaultError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let rtn = match method {
            VaultMethod::Put => {
                let input: VaultPutInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let bucket = system_api
                    .node_drop(RENodeId::Bucket(input.bucket.0))
                    .map_err(InvokeError::Downstream)?
                    .into();
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                vault
                    .put(bucket)
                    .map_err(|e| InvokeError::Error(VaultError::ResourceOperationError(e)))?;
                Ok(ScryptoValue::from_typed(&()))
            }
            VaultMethod::Take => {
                let input: VaultTakeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                let container = vault.take(input.amount)?;
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            VaultMethod::LockFee | VaultMethod::LockContingentFee => {
                let input: VaultLockFeeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;

                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();

                // Check resource and take amount
                if vault.resource_address() != RADIX_TOKEN {
                    return Err(InvokeError::Error(VaultError::LockFeeNotRadixToken));
                }

                // Take fee from the vault
                let fee = vault
                    .take(input.amount)
                    .map_err(|_| InvokeError::Error(VaultError::LockFeeInsufficientBalance))?;

                // Refill fee reserve
                let changes = system_api
                    .lock_fee(
                        vault_id,
                        fee,
                        matches!(method, VaultMethod::LockContingentFee),
                    )
                    .map_err(InvokeError::Downstream)?;

                // Return changes
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                vault
                    .borrow_resource_mut()
                    .put(changes)
                    .expect("Failed to return fee changes to a locking-fee vault");

                Ok(ScryptoValue::from_typed(&()))
            }
            VaultMethod::TakeNonFungibles => {
                let input: VaultTakeNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                let container = vault.take_non_fungibles(&input.non_fungible_ids)?;
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            VaultMethod::GetAmount => {
                let _: VaultGetAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                let amount = vault.total_amount();
                Ok(ScryptoValue::from_typed(&amount))
            }
            VaultMethod::GetResourceAddress => {
                let _: VaultGetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                let resource_address = vault.resource_address();
                Ok(ScryptoValue::from_typed(&resource_address))
            }
            VaultMethod::GetNonFungibleIds => {
                let _: VaultGetNonFungibleIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                let ids = vault
                    .total_ids()
                    .map_err(|e| InvokeError::Error(VaultError::ResourceOperationError(e)))?;
                Ok(ScryptoValue::from_typed(&ids))
            }
            VaultMethod::CreateProof => {
                let _: VaultCreateProofInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                let proof = vault
                    .create_proof(ResourceContainerId::Vault(vault_id))
                    .map_err(|e| InvokeError::Error(VaultError::ProofError(e)))?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            VaultMethod::CreateProofByAmount => {
                let input: VaultCreateProofByAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                let proof = vault
                    .create_proof_by_amount(input.amount, ResourceContainerId::Vault(vault_id))
                    .map_err(|e| InvokeError::Error(VaultError::ProofError(e)))?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            VaultMethod::CreateProofByIds => {
                let input: VaultCreateProofByIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::Vault(vault_id.clone()))
                    .map_err(InvokeError::Downstream)?;
                let vault = node_ref.vault_mut();
                let proof = vault
                    .create_proof_by_ids(&input.ids, ResourceContainerId::Vault(vault_id))
                    .map_err(|e| InvokeError::Error(VaultError::ProofError(e)))?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
        }?;

        Ok(rtn)
    }
}
