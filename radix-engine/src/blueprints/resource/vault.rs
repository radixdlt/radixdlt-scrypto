use crate::blueprints::resource::*;
use crate::errors::{ApplicationError, InterpreterError};
use crate::errors::{InvokeError, RuntimeError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::CostingError;
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{RENodeId, SubstateOffset, VaultOffset};
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;
use sbor::rust::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct VaultSubstate(pub Resource);

impl VaultSubstate {
    pub fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }
}

#[derive(Debug)]
pub struct VaultRuntimeSubstate {
    resource: Rc<RefCell<LockableResource>>,
}

impl VaultRuntimeSubstate {
    pub fn clone_to_persisted(&self) -> VaultSubstate {
        let lockable_resource = self.borrow_resource();
        if lockable_resource.is_locked() {
            // We keep resource containers in Rc<RefCell> for all concrete resource containers, like Bucket, Vault and Worktop.
            // When extracting the resource within a container, there should be no locked resource.
            // It should have failed the Rc::try_unwrap() check.
            panic!("Attempted to convert resource container with locked resource");
        }
        let resource = match lockable_resource.deref() {
            LockableResource::Fungible {
                resource_address,
                divisibility,
                liquid_amount,
                ..
            } => Resource::Fungible {
                resource_address: resource_address.clone(),
                divisibility: divisibility.clone(),
                amount: liquid_amount.clone(),
            },
            LockableResource::NonFungible {
                resource_address,
                liquid_ids,
                id_type,
                ..
            } => Resource::NonFungible {
                resource_address: resource_address.clone(),
                ids: liquid_ids.clone(),
                id_type: *id_type,
            },
        };

        VaultSubstate(resource)
    }

    pub fn to_persisted(self) -> Result<VaultSubstate, ResourceOperationError> {
        Rc::try_unwrap(self.resource)
            .map_err(|_| ResourceOperationError::ResourceLocked)
            .map(|c| c.into_inner())
            .map(Into::into)
            .map(|r| VaultSubstate(r))
    }

    pub fn new(resource: Resource) -> Self {
        Self {
            resource: Rc::new(RefCell::new(resource.into())),
        }
    }

    pub fn put(&mut self, other: BucketSubstate) -> Result<(), ResourceOperationError> {
        self.borrow_resource_mut().put(other.resource()?)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Resource, InvokeError<VaultError>> {
        let resource = self
            .borrow_resource_mut()
            .take_by_amount(amount)
            .map_err(|e| InvokeError::SelfError(VaultError::ResourceOperationError(e)))?;
        Ok(resource)
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<Resource, InvokeError<VaultError>> {
        let resource = self
            .borrow_resource_mut()
            .take_by_ids(ids)
            .map_err(|e| InvokeError::SelfError(VaultError::ResourceOperationError(e)))?;
        Ok(resource)
    }

    pub fn create_proof(
        &mut self,
        container_id: ResourceContainerId,
    ) -> Result<ProofSubstate, ProofError> {
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), container_id)
            }
            ResourceType::NonFungible { .. } => self.create_proof_by_ids(
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
    ) -> Result<ProofSubstate, ProofError> {
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
        ProofSubstate::new(
            self.resource_address(),
            self.resource_type(),
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn create_proof_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
        container_id: ResourceContainerId,
    ) -> Result<ProofSubstate, ProofError> {
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
        ProofSubstate::new(
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

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleLocalId>, ResourceOperationError> {
        self.borrow_resource().total_ids()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_resource().is_locked()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_resource().is_empty()
    }

    pub fn borrow_resource(&self) -> Ref<LockableResource> {
        self.resource.borrow()
    }

    pub fn borrow_resource_mut(&mut self) -> RefMut<LockableResource> {
        self.resource.borrow_mut()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum VaultError {
    InvalidRequestData(DecodeError),
    ResourceOperationError(ResourceOperationError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ProofError(ProofError),
    CouldNotCreateProof,
    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
    LockFeeRepayFailure(CostingError),
}

pub struct VaultBlueprint;

impl VaultBlueprint {
    fn take_internal<Y>(
        receiver: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::MUTABLE,
        )?;

        let container = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.take(amount)?
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(container)),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();
        Ok(Bucket(bucket_id))
    }

    fn take_non_fungibles_internal<Y>(
        receiver: RENodeId,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::MUTABLE,
        )?;

        let container = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.take_non_fungibles(&non_fungible_local_ids)?
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(container)),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(Bucket(bucket_id))
    }

    pub(crate) fn take<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: VaultTakeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket = Self::take_internal(receiver, input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    pub(crate) fn take_non_fungibles<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: VaultTakeNonFungiblesInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket =
            Self::take_non_fungibles_internal(receiver, input.non_fungible_local_ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    pub(crate) fn put<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: VaultPutInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::MUTABLE,
        )?;

        let bucket = api
            .kernel_drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();

        let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
        let vault = substate_mut.vault();
        vault.put(bucket).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::ResourceOperationError(e),
            ))
        })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn get_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let _input: VaultGetAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::read_only(),
        )?;

        let vault: &VaultRuntimeSubstate = api.kernel_get_substate_ref(vault_handle)?;
        let amount = vault.total_amount();

        Ok(IndexedScryptoValue::from_typed(&amount))
    }

    pub(crate) fn get_resource_address<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let _input: VaultGetResourceAddressInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::read_only(),
        )?;

        let vault: &VaultRuntimeSubstate = api.kernel_get_substate_ref(vault_handle)?;
        let resource_address = vault.resource_address();

        Ok(IndexedScryptoValue::from_typed(&resource_address))
    }

    pub(crate) fn get_non_fungible_local_ids<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let _input: VaultGetNonFungibleLocalIdsInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::read_only(),
        )?;

        let vault: &VaultRuntimeSubstate = api.kernel_get_substate_ref(vault_handle)?;
        let ids = vault.total_ids().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::ResourceOperationError(e),
            ))
        })?;

        Ok(IndexedScryptoValue::from_typed(&ids))
    }

    pub(crate) fn lock_fee<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: VaultLockFeeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        // Take by amount
        let fee = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();

            // Check resource and take amount
            if vault.resource_address() != RADIX_TOKEN {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::VaultError(VaultError::LockFeeNotRadixToken),
                ));
            }

            // Take fee from the vault
            vault.take(input.amount).map_err(|_| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::LockFeeInsufficientBalance,
                ))
            })?
        };

        // Credit cost units
        let changes: Resource = api.credit_cost_units(receiver.into(), fee, input.contingent)?;

        // Keep changes
        {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.put(BucketSubstate::new(changes)).map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::ResourceOperationError(e),
                ))
            })?;
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn recall<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: VaultRecallInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket = Self::take_internal(receiver, input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    pub(crate) fn recall_non_fungibles<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: VaultRecallNonFungiblesInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket =
            Self::take_non_fungibles_internal(receiver, input.non_fungible_local_ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    pub(crate) fn create_proof<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let _input: VaultCreateProofInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::MUTABLE,
        )?;

        let proof = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof(ResourceContainerId::Vault(receiver.into()))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn create_proof_by_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: VaultCreateProofByAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::MUTABLE,
        )?;

        let proof = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof_by_amount(input.amount, ResourceContainerId::Vault(receiver.into()))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn create_proof_by_ids<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: VaultCreateProofByIdsInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Vault),
            LockFlags::MUTABLE,
        )?;

        let proof = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof_by_ids(&input.ids, ResourceContainerId::Vault(receiver.into()))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }
}
