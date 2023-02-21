use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::CostingError;
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{RENodeId, SubstateOffset, VaultOffset};
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct VaultInfoSubstate {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum VaultError {
    InvalidRequestData(DecodeError),
    ResourceError(ResourceError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ProofError(ProofError),
    CouldNotCreateProof,
    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
    LockFeeRepayFailure(CostingError),

    NonFungibleOperationOnFungible,
    MismatchingFungibility,
}

pub struct VaultNode;

impl VaultNode {
    pub(crate) fn get_info<Y>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<(ResourceAddress, ResourceType), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Info),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let resource_address = substate_ref.vault_info().resource_address;
        let resource_type = substate_ref.vault_info().resource_type;
        api.kernel_drop_lock(handle)?;
        Ok((resource_address, resource_type))
    }

    pub(crate) fn is_locked<Y>(node_id: RENodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LockedFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let locked = substate_ref.vault_locked_fungible().is_locked();
                api.kernel_drop_lock(handle)?;
                Ok(locked)
            }
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LockedNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let locked = substate_ref.vault_locked_non_fungible().is_locked();
                api.kernel_drop_lock(handle)?;
                Ok(locked)
            }
        }
    }

    pub(crate) fn amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LiquidFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let amount = substate_ref.vault_liquid_fungible().amount();
                api.kernel_drop_lock(handle)?;
                Ok(amount)
            }
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let amount = substate_ref.vault_liquid_non_fungible().amount();
                api.kernel_drop_lock(handle)?;
                Ok(amount)
            }
        }
    }

    pub(crate) fn non_fungible_ids<Y>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationOnFungible),
            )),
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let ids = substate_ref.vault_liquid_non_fungible().ids().clone();
                api.kernel_drop_lock(handle)?;
                Ok(ids)
            }
        }
    }

    pub(crate) fn take<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LiquidFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let taken = substate_ref
                    .vault_liquid_fungible()
                    .take_by_amount(amount)
                    .map_err(VaultError::ResourceError)
                    .map_err(|e| RuntimeError::ApplicationError(ApplicationError::VaultError(e)))?;
                api.kernel_drop_lock(handle)?;
                Ok(LiquidResource::Fungible(taken))
            }
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let taken = substate_ref
                    .vault_liquid_non_fungible()
                    .take_by_amount(amount)
                    .map_err(VaultError::ResourceError)
                    .map_err(|e| RuntimeError::ApplicationError(ApplicationError::VaultError(e)))?;
                api.kernel_drop_lock(handle)?;
                Ok(LiquidResource::NonFungible(taken))
            }
        }
    }

    pub(crate) fn take_non_fungibles<Y>(
        node_id: RENodeId,
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationOnFungible),
            )),
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let taken = substate_ref
                    .vault_liquid_non_fungible()
                    .take_by_ids(ids)
                    .map_err(VaultError::ResourceError)
                    .map_err(|e| RuntimeError::ApplicationError(ApplicationError::VaultError(e)))?;
                api.kernel_drop_lock(handle)?;
                Ok(taken)
            }
        }
    }

    pub(crate) fn put<Y>(
        node_id: RENodeId,
        resource: LiquidResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LiquidFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let taken = substate_ref
                    .vault_liquid_fungible()
                    .put(
                        resource
                            .into_fungible()
                            .ok_or(RuntimeError::ApplicationError(
                                ApplicationError::VaultError(VaultError::MismatchingFungibility),
                            ))?,
                    )
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::VaultError(
                            VaultError::ResourceError(e),
                        ))
                    })?;
                api.kernel_drop_lock(handle)?;
                Ok(())
            }
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let taken =
                    substate_ref
                        .vault_liquid_non_fungible()
                        .put(resource.into_non_fungibles().ok_or(
                            RuntimeError::ApplicationError(ApplicationError::VaultError(
                                VaultError::MismatchingFungibility,
                            )),
                        )?)
                        .map_err(|e| {
                            RuntimeError::ApplicationError(ApplicationError::VaultError(
                                VaultError::ResourceError(e),
                            ))
                        })?;
                api.kernel_drop_lock(handle)?;
                Ok(())
            }
        }
    }
}

pub struct VaultBlueprint;

impl VaultBlueprint {
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

        // Take
        let taken = VaultNode::take(receiver, input.amount, api)?;
        let info = BucketInfoSubstate {
            resource_address: taken.resource_address(),
            resource_type: taken.resource_type(),
        };

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            match taken {
                LiquidResource::Fungible(f) => RENodeInit::FungibleBucket(info, f),
                LiquidResource::NonFungible(nf) => RENodeInit::NonFungibleBucket(info, nf),
            },
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
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

        // Take
        let taken = VaultNode::take_non_fungibles(receiver, &input.non_fungible_local_ids, api)?;
        let info = BucketInfoSubstate {
            resource_address: taken.resource_address(),
            resource_type: taken.resource_type(),
        };

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::NonFungibleBucket(info, taken),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
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

        // Drop other bucket
        let other_bucket: LiquidResource = api
            .kernel_drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();

        // Put
        VaultNode::put(receiver, other_bucket, api)?;

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

        let amount: Decimal = VaultNode::amount(receiver, api)?;

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

        let resource_address: ResourceAddress = VaultNode::get_info(receiver, api)?.0;

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

        let ids: BTreeSet<NonFungibleLocalId> = VaultNode::non_fungible_ids(receiver, api)?;

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
            SubstateOffset::Vault(VaultOffset::Info),
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        // Take by amount
        let fee = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault_info();

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
            let vault = substate_mut.vault_info();
            vault.put(BucketSubstate::new(changes)).map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::ResourceError(e),
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
            SubstateOffset::Vault(VaultOffset::Info),
            LockFlags::MUTABLE,
        )?;

        let proof = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault_info();
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
            SubstateOffset::Vault(VaultOffset::Info),
            LockFlags::MUTABLE,
        )?;

        let proof = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault_info();
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
            SubstateOffset::Vault(VaultOffset::Info),
            LockFlags::MUTABLE,
        )?;

        let proof = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault_info();
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
