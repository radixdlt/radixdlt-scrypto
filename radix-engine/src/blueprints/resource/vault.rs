use core::ops::SubAssign;

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
    ProofError(ProofError),
    NonFungibleOperationNotSupported,
    MismatchingFungibility,

    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
    LockFeeRepayFailure(CostingError),
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

    pub(crate) fn liquid_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
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

    pub(crate) fn locked_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
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
                let amount = substate_ref.vault_locked_fungible().amount();
                api.kernel_drop_lock(handle)?;
                Ok(amount)
            }
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LockedNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let amount = substate_ref.vault_locked_non_fungible().amount();
                api.kernel_drop_lock(handle)?;
                Ok(amount)
            }
        }
    }

    pub(crate) fn is_locked<Y>(node_id: RENodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        Ok(!Self::locked_amount(node_id, api)?.is_zero())
    }

    pub(crate) fn liquid_non_fungible_ids<Y>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
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
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
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
        if resource.is_empty() {
            return Ok(());
        }

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

    pub(crate) fn lock_amount<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<ProofSubstate, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let (resource_address, resource_type) = Self::get_info(node_id, api)?;
        check_amount(amount, resource_type.divisibility()).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;

        match resource_type {
            ResourceType::Fungible { divisibility } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LockedFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let locked = substate_ref.vault_locked_fungible();
                let max_locked = locked.amount();

                // Take from liquid if needed
                if amount > max_locked {
                    let delta = amount - max_locked;
                    VaultNode::take(node_id, delta, api)?;
                }

                // Increase lock count
                locked.amounts.entry(amount).or_default().add_assign(1);

                // Issue proof
                Ok(ProofSubstate::Fungible(
                    FungibleProof::new(
                        resource_address,
                        amount,
                        btreemap!(
                            LocalRef::Vault(node_id.into()) => amount
                        ),
                    )
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::VaultError(
                            VaultError::ProofError(e),
                        ))
                    })?,
                ))
            }
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LockedNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let locked = substate_ref.vault_locked_non_fungible();
                let max_locked: Decimal = locked.ids.len().into();

                // Take from liquid if needed
                if amount > max_locked {
                    let delta = amount - max_locked;
                    let resource = VaultNode::take(node_id, delta, api)?;
                    let non_fungibles = resource
                        .into_non_fungibles()
                        .expect("Should be non-fungibles");
                    for nf in non_fungibles.into_ids() {
                        locked.ids.insert(nf, 0);
                    }
                }

                // Increase lock count
                let n: usize = amount
                    .to_string()
                    .parse()
                    .expect("Failed to convert amount to usize");
                let ids_for_proof: BTreeSet<NonFungibleLocalId> =
                    locked.ids.keys().cloned().into_iter().take(n).collect();
                for id in &ids_for_proof {
                    locked.ids.get_mut(id).unwrap().add_assign(1);
                }

                // Issue proof
                Ok(ProofSubstate::NonFungible(
                    NonFungibleProof::new(
                        resource_address,
                        ids_for_proof,
                        btreemap!(
                            LocalRef::Vault(node_id.into()) => ids_for_proof
                        ),
                    )
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::VaultError(
                            VaultError::ProofError(e),
                        ))
                    })?,
                ))
            }
        }
    }

    pub(crate) fn lock_non_fungibles<Y>(
        node_id: RENodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<ProofSubstate, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let (resource_address, resource_type) = Self::get_info(node_id, api)?;

        match resource_type {
            ResourceType::Fungible { divisibility } => Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
            )),
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LockedNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let locked = substate_ref.vault_locked_non_fungible();

                // Take from liquid if needed
                let delta: BTreeSet<NonFungibleLocalId> = ids
                    .iter()
                    .cloned()
                    .filter(|id| !locked.ids.contains_key(id))
                    .collect();
                VaultNode::take_non_fungibles(node_id, &delta, api)?;

                // Increase lock count
                for id in &ids {
                    locked.ids.get_mut(id).unwrap().add_assign(1);
                }

                // Issue proof
                Ok(ProofSubstate::NonFungible(
                    NonFungibleProof::new(
                        resource_address,
                        ids,
                        btreemap!(
                            LocalRef::Vault(node_id.into()) => ids
                        ),
                    )
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::VaultError(
                            VaultError::ProofError(e),
                        ))
                    })?,
                ))
            }
        }
    }

    pub(crate) fn unlock_amount<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let (resource_address, resource_type) = Self::get_info(node_id, api)?;

        match resource_type {
            ResourceType::Fungible { divisibility } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LockedFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let locked = substate_ref.vault_locked_fungible();

                let max_locked = locked.amount();
                locked
                    .amounts
                    .get_mut(&amount)
                    .expect("Attempted to unlock an amount that is not locked in container")
                    .sub_assign(1);

                let delta = max_locked - locked.amount();
                VaultNode::put(
                    node_id,
                    LiquidResource::Fungible(LiquidFungibleResource::new(
                        resource_address,
                        divisibility,
                        delta,
                    )),
                    api,
                )?;

                Ok(())
            }
            ResourceType::NonFungible { id_type } => {
                panic!("Attempted to unlock amount on non-fungibles")
            }
        }
    }

    pub(crate) fn unlock_non_fungibles<Y>(
        node_id: RENodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let (resource_address, resource_type) = Self::get_info(node_id, api)?;

        match resource_type {
            ResourceType::Fungible { divisibility } => {
                panic!("Attempted to unlock non-fungibles on fungible")
            }
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::LockedNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let locked = substate_ref.vault_locked_non_fungible();

                let mut liquid_non_fungibles = BTreeSet::<NonFungibleLocalId>::new();
                for id in ids {
                    let cnt = locked
                        .ids
                        .remove(&id)
                        .expect("Attempted to unlock non-fungible that was not locked");
                    if cnt > 1 {
                        locked.ids.insert(id, cnt - 1);
                    } else {
                        liquid_non_fungibles.insert(id);
                    }
                }

                VaultNode::put(
                    node_id,
                    LiquidResource::NonFungible(LiquidNonFungibleResource::new(
                        resource_address,
                        id_type,
                        liquid_non_fungibles,
                    )),
                    api,
                )?;

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

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            match taken {
                LiquidResource::Fungible(f) => RENodeInit::FungibleBucket(f),
                LiquidResource::NonFungible(nf) => RENodeInit::NonFungibleBucket(nf),
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

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::NonFungibleBucket(taken),
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

        let amount: Decimal = VaultNode::liquid_amount(receiver, api)?;

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

        let ids: BTreeSet<NonFungibleLocalId> = VaultNode::liquid_non_fungible_ids(receiver, api)?;

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

        let resource_address = VaultNode::get_info(receiver, api)?.0;

        // Check resource address
        if resource_address != RADIX_TOKEN {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::LockFeeNotRadixToken),
            ));
        }

        // Lock the substate (with special flags)
        let vault_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Info),
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        // Take by amount
        let fee = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault_liquid_fungible();

            // Take fee from the vault
            vault.take_by_amount(input.amount).map_err(|_| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::LockFeeInsufficientBalance,
                ))
            })?
        };

        // Credit cost units
        let changes = api.credit_cost_units(receiver.into(), fee, input.contingent)?;

        // Keep changes
        {
            let mut substate_mut = api.kernel_get_substate_ref_mut(vault_handle)?;
            let vault = substate_mut.vault_liquid_fungible();
            vault.put(changes).expect("Failed to put fee changes");
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

        let taken = VaultNode::take(receiver, input.amount, api)?;

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            match taken {
                LiquidResource::Fungible(f) => RENodeInit::FungibleBucket(f),
                LiquidResource::NonFungible(nf) => RENodeInit::NonFungibleBucket(nf),
            },
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
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

        let taken = VaultNode::take_non_fungibles(receiver, &input.non_fungible_local_ids, api)?;

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::NonFungibleBucket(taken),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
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

        let amount =
            VaultNode::locked_amount(receiver, api)? + VaultNode::liquid_amount(receiver, api)?;
        let proof = VaultNode::lock_amount(receiver, amount, api)?;

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

        let proof = VaultNode::lock_amount(receiver, input.amount, api)?;

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

        let proof = VaultNode::lock_non_fungibles(receiver, input.ids, api)?;

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }
}
