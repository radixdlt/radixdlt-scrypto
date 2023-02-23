use core::ops::SubAssign;

use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::heap::{DroppedBucket, DroppedBucketResource};
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::CostingError;
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{RENodeId, SubstateOffset, VaultOffset};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum VaultError {
    InvalidRequestData(DecodeError),

    ResourceError(ResourceError),
    ProofError(ProofError),
    NonFungibleOperationNotSupported,
    MismatchingResource,
    InvalidAmount,

    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
    LockFeeRepayFailure(CostingError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct VaultInfoSubstate {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
}

impl VaultInfoSubstate {
    pub fn of<Y>(node_id: RENodeId, api: &mut Y) -> Result<Self, RuntimeError>
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
        let info = substate_ref.vault_info().clone();
        api.kernel_drop_lock(handle)?;
        Ok(info)
    }
}

pub struct FungibleVault;

impl FungibleVault {
    pub fn liquid_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
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

    pub fn locked_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
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

    pub fn is_locked<Y>(node_id: RENodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        Ok(!Self::locked_amount(node_id, api)?.is_zero())
    }

    pub fn take<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LiquidFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let taken = substate_ref
            .vault_liquid_fungible()
            .take_by_amount(amount)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::ResourceError(e),
                ))
            })?;
        api.kernel_drop_lock(handle)?;
        Ok(taken)
    }

    pub fn put<Y>(
        node_id: RENodeId,
        resource: LiquidFungibleResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LiquidFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        substate_ref
            .vault_liquid_fungible()
            .put(resource)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::ResourceError(e),
                ))
            })?;
        api.kernel_drop_lock(handle)?;
        Ok(())
    }

    // protected method
    pub fn lock_amount<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<FungibleProof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LockedFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let mut locked = substate_ref.vault_locked_fungible();
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            FungibleVault::take(node_id, delta, api)?;
        }

        // Increase lock count
        substate_ref = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
        locked = substate_ref.vault_locked_fungible();
        locked.amounts.entry(amount).or_default().add_assign(1);

        // Issue proof
        Ok(FungibleProof::new(
            amount,
            btreemap!(
                LocalRef::Vault(node_id.into()) => amount
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
        })?)
    }

    // protected method
    pub fn unlock_amount<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LockedFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let locked = substate_ref.vault_locked_fungible();

        let max_locked = locked.amount();
        locked
            .amounts
            .get_mut(&amount)
            .expect("Attempted to unlock an amount that is not locked in container")
            .sub_assign(1);

        let delta = max_locked - locked.amount();
        FungibleVault::put(node_id, LiquidFungibleResource::new(delta), api)
    }
}

pub struct NonFungibleVault;

impl NonFungibleVault {
    pub fn liquid_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
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

    pub fn locked_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
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

    pub fn is_locked<Y>(node_id: RENodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        Ok(!Self::locked_amount(node_id, api)?.is_zero())
    }

    pub fn liquid_non_fungible_local_ids<Y>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
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

    pub fn take<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let taken = substate_ref
            .vault_liquid_non_fungible()
            .take_by_amount(amount)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::ResourceError(e),
                ))
            })?;
        api.kernel_drop_lock(handle)?;
        Ok(taken)
    }

    pub fn take_non_fungibles<Y>(
        node_id: RENodeId,
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let taken = substate_ref
            .vault_liquid_non_fungible()
            .take_by_ids(ids)
            .map_err(VaultError::ResourceError)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::VaultError(e)))?;
        api.kernel_drop_lock(handle)?;
        Ok(taken)
    }

    pub fn put<Y>(
        node_id: RENodeId,
        resource: LiquidNonFungibleResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        substate_ref
            .vault_liquid_non_fungible()
            .put(resource)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::ResourceError(e),
                ))
            })?;
        api.kernel_drop_lock(handle)?;
        Ok(())
    }

    // protected method
    pub fn lock_amount<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<NonFungibleProof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LockedNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let mut locked = substate_ref.vault_locked_non_fungible();
        let max_locked: Decimal = locked.ids.len().into();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            let resource = NonFungibleVault::take(node_id, delta, api)?;

            substate_ref = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
            locked = substate_ref.vault_locked_non_fungible();
            for nf in resource.into_ids() {
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
        Ok(NonFungibleProof::new(
            ids_for_proof.clone(),
            btreemap!(
                LocalRef::Vault(node_id.into()) => ids_for_proof
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
        })?)
    }

    // protected method
    pub fn lock_non_fungibles<Y>(
        node_id: RENodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<NonFungibleProof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LockedNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let mut locked = substate_ref.vault_locked_non_fungible();

        // Take from liquid if needed
        let delta: BTreeSet<NonFungibleLocalId> = ids
            .iter()
            .cloned()
            .filter(|id| !locked.ids.contains_key(id))
            .collect();
        NonFungibleVault::take_non_fungibles(node_id, &delta, api)?;

        // Increase lock count
        substate_ref = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
        locked = substate_ref.vault_locked_non_fungible();
        for id in &ids {
            locked.ids.get_mut(id).unwrap().add_assign(1);
        }

        // Issue proof
        Ok(NonFungibleProof::new(
            ids.clone(),
            btreemap!(
                LocalRef::Vault(node_id.into()) => ids
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
        })?)
    }

    // protected method
    pub fn unlock_non_fungibles<Y>(
        node_id: RENodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::LockedNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
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

        NonFungibleVault::put(
            node_id,
            LiquidNonFungibleResource::new(liquid_non_fungibles),
            api,
        )
    }
}

pub struct VaultBlueprint;

impl VaultBlueprint {
    pub fn take<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultTakeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        // Check amount
        let info = VaultInfoSubstate::of(receiver, api)?;
        if !info.resource_type.check_amount(input.amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let node_id = if info.resource_type.is_fungible() {
            // Take
            let taken = FungibleVault::take(receiver, input.amount, api)?;

            // Create node
            let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::FungibleBucket(
                    BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    },
                    taken,
                ),
                BTreeMap::new(),
            )?;

            node_id
        } else {
            // Take
            let taken = NonFungibleVault::take(receiver, input.amount, api)?;

            // Create node
            let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::NonFungibleBucket(
                    BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    },
                    taken,
                ),
                BTreeMap::new(),
            )?;

            node_id
        };
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
    }

    pub fn take_non_fungibles<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultTakeNonFungiblesInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = VaultInfoSubstate::of(receiver, api)?;

        if info.resource_type.is_fungible() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
            ));
        } else {
            // Take
            let taken =
                NonFungibleVault::take_non_fungibles(receiver, &input.non_fungible_local_ids, api)?;

            // Create node
            let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::NonFungibleBucket(
                    BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    },
                    taken,
                ),
                BTreeMap::new(),
            )?;
            let bucket_id = node_id.into();

            Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
        }
    }

    pub fn put<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultPutInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        // Drop other bucket
        let other_bucket: DroppedBucket = api
            .kernel_drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();

        // Check resource address
        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_address != other_bucket.info.resource_address {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::MismatchingResource),
            ));
        }

        // Put
        match other_bucket.resource {
            DroppedBucketResource::Fungible(r) => {
                FungibleVault::put(receiver, r, api)?;
            }
            DroppedBucketResource::NonFungible(r) => {
                NonFungibleVault::put(receiver, r, api)?;
            }
        }
        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn get_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: VaultGetAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        let amount = if info.resource_type.is_fungible() {
            FungibleVault::liquid_amount(receiver, api)?
        } else {
            NonFungibleVault::liquid_amount(receiver, api)?
        };

        Ok(IndexedScryptoValue::from_typed(&amount))
    }

    pub fn get_resource_address<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: VaultGetResourceAddressInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = VaultInfoSubstate::of(receiver, api)?;

        Ok(IndexedScryptoValue::from_typed(&info.resource_address))
    }

    pub fn get_non_fungible_local_ids<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: VaultGetNonFungibleLocalIdsInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_type.is_fungible() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
            ));
        } else {
            let ids: BTreeSet<NonFungibleLocalId> =
                NonFungibleVault::liquid_non_fungible_local_ids(receiver, api)?;
            Ok(IndexedScryptoValue::from_typed(&ids))
        }
    }

    pub fn lock_fee<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultLockFeeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        // Check resource address
        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_address != RADIX_TOKEN {
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

    pub fn recall<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultRecallInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        let node_id = if info.resource_type.is_fungible() {
            let taken = FungibleVault::take(receiver, input.amount, api)?;
            let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::FungibleBucket(
                    BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    },
                    taken,
                ),
                BTreeMap::new(),
            )?;
            node_id
        } else {
            let taken = NonFungibleVault::take(receiver, input.amount, api)?;
            let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::NonFungibleBucket(
                    BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    },
                    taken,
                ),
                BTreeMap::new(),
            )?;
            node_id
        };
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
    }

    pub fn recall_non_fungibles<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultRecallNonFungiblesInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_type.is_fungible() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
            ));
        } else {
            let taken =
                NonFungibleVault::take_non_fungibles(receiver, &input.non_fungible_local_ids, api)?;

            let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::NonFungibleBucket(
                    BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    },
                    taken,
                ),
                BTreeMap::new(),
            )?;
            let bucket_id = node_id.into();

            Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
        }
    }

    pub fn create_proof<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: VaultCreateProofInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        let node_id = if info.resource_type.is_fungible() {
            let amount = FungibleVault::locked_amount(receiver, api)?
                + FungibleVault::liquid_amount(receiver, api)?;
            let proof = FungibleVault::lock_amount(receiver, amount, api)?;

            let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::FungibleProof(
                    ProofInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: false,
                    },
                    proof,
                ),
                BTreeMap::new(),
            )?;
            node_id
        } else {
            let amount = NonFungibleVault::locked_amount(receiver, api)?
                + NonFungibleVault::liquid_amount(receiver, api)?;
            let proof = NonFungibleVault::lock_amount(receiver, amount, api)?;

            let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::NonFungibleProof(
                    ProofInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: false,
                    },
                    proof,
                ),
                BTreeMap::new(),
            )?;
            node_id
        };

        let proof_id = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub fn create_proof_by_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultCreateProofByAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        let node_id = if info.resource_type.is_fungible() {
            let proof = FungibleVault::lock_amount(receiver, input.amount, api)?;

            let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::FungibleProof(
                    ProofInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: false,
                    },
                    proof,
                ),
                BTreeMap::new(),
            )?;
            node_id
        } else {
            let proof = NonFungibleVault::lock_amount(receiver, input.amount, api)?;

            let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::NonFungibleProof(
                    ProofInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: false,
                    },
                    proof,
                ),
                BTreeMap::new(),
            )?;
            node_id
        };

        let proof_id = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub fn create_proof_by_ids<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultCreateProofByIdsInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = VaultInfoSubstate::of(receiver, api)?;

        if info.resource_type.is_fungible() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
            ));
        } else {
            let proof = NonFungibleVault::lock_non_fungibles(receiver, input.ids, api)?;

            let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::NonFungibleProof(
                    ProofInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: false,
                    },
                    proof,
                ),
                BTreeMap::new(),
            )?;

            let proof_id = node_id.into();
            Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
        }
    }

    //===================
    // Protected method
    //===================

    // FIXME: set up auth

    pub fn lock_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultLockAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        FungibleVault::lock_amount(receiver, input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn lock_non_fungibles<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultLockNonFungiblesInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        NonFungibleVault::lock_non_fungibles(receiver, input.local_ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn unlock_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultUnlockAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        FungibleVault::unlock_amount(receiver, input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn unlock_non_fungibles<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultUnlockNonFungiblesInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        NonFungibleVault::unlock_non_fungibles(receiver, input.local_ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
