use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::heap::{DroppedBucket, DroppedBucketResource};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::CostingError;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::{types::*, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum VaultError {
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
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::Info),
            LockFlags::read_only(),
        )?;
        let substate_ref: &VaultInfoSubstate = api.kernel_get_substate_ref(handle)?;
        let info = substate_ref.clone();
        api.sys_drop_lock(handle)?;
        Ok(info)
    }
}

pub struct FungibleVault;

impl FungibleVault {
    pub fn liquid_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LiquidFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref: &LiquidFungibleResource = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn locked_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LockedFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref: &LockedFungibleResource = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn is_locked<Y>(node_id: RENodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        Ok(!Self::locked_amount(node_id, api)?.is_zero())
    }

    pub fn take<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LiquidFungible),
            LockFlags::MUTABLE,
        )?;
        let substate_ref: &mut LiquidFungibleResource = api.kernel_get_substate_ref_mut(handle)?;
        let taken = substate_ref.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, WithdrawResourceEvent::Amount(amount))?;

        Ok(taken)
    }

    pub fn put<Y>(
        node_id: RENodeId,
        resource: LiquidFungibleResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let event = DepositResourceEvent::Amount(resource.amount());

        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LiquidFungible),
            LockFlags::MUTABLE,
        )?;
        let substate_ref: &mut LiquidFungibleResource = api.kernel_get_substate_ref_mut(handle)?;
        substate_ref.put(resource).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(())
    }

    // protected method
    pub fn lock_amount<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<FungibleProof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LockedFungible),
            LockFlags::MUTABLE,
        )?;
        let mut locked: &mut LockedFungibleResource = api.kernel_get_substate_ref_mut(handle)?;
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            FungibleVault::take(node_id, delta, api)?;
        }

        // Increase lock count
        locked = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LockedFungible),
            LockFlags::MUTABLE,
        )?;
        let locked: &mut LockedFungibleResource = api.kernel_get_substate_ref_mut(handle)?;

        let max_locked = locked.amount();
        let cnt = locked
            .amounts
            .remove(&amount)
            .expect("Attempted to unlock an amount that is not locked");
        if cnt > 1 {
            locked.amounts.insert(amount, cnt - 1);
        }

        let delta = max_locked - locked.amount();
        FungibleVault::put(node_id, LiquidFungibleResource::new(delta), api)
    }
}

pub struct NonFungibleVault;

impl NonFungibleVault {
    pub fn liquid_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref: &LiquidNonFungibleResource = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn locked_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LockedNonFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref: &LockedNonFungibleResource = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn is_locked<Y>(node_id: RENodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        Ok(!Self::locked_amount(node_id, api)?.is_zero())
    }

    pub fn liquid_non_fungible_local_ids<Y>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref: &LiquidNonFungibleResource = api.kernel_get_substate_ref(handle)?;
        let ids = substate_ref.ids().clone();
        api.sys_drop_lock(handle)?;
        Ok(ids)
    }

    pub fn locked_non_fungible_local_ids<Y>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LockedNonFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref: &LockedNonFungibleResource = api.kernel_get_substate_ref(handle)?;
        let ids = substate_ref.ids();
        api.sys_drop_lock(handle)?;
        Ok(ids)
    }

    pub fn take<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
            LockFlags::MUTABLE,
        )?;
        let substate_ref: &mut LiquidNonFungibleResource =
            api.kernel_get_substate_ref_mut(handle)?;
        let taken = substate_ref.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, WithdrawResourceEvent::Amount(amount))?;

        Ok(taken)
    }

    pub fn take_non_fungibles<Y>(
        node_id: RENodeId,
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
            LockFlags::MUTABLE,
        )?;
        let substate_ref: &mut LiquidNonFungibleResource =
            api.kernel_get_substate_ref_mut(handle)?;
        let taken = substate_ref
            .take_by_ids(ids)
            .map_err(VaultError::ResourceError)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::VaultError(e)))?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, WithdrawResourceEvent::Ids(ids.clone()))?;

        Ok(taken)
    }

    pub fn put<Y>(
        node_id: RENodeId,
        resource: LiquidNonFungibleResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let event = DepositResourceEvent::Ids(resource.ids().clone());

        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
            LockFlags::MUTABLE,
        )?;
        let substate_ref: &mut LiquidNonFungibleResource =
            api.kernel_get_substate_ref_mut(handle)?;
        substate_ref.put(resource).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(())
    }

    // protected method
    pub fn lock_amount<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<NonFungibleProof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LockedNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut locked: &mut LockedNonFungibleResource = api.kernel_get_substate_ref_mut(handle)?;
        let max_locked: Decimal = locked.ids.len().into();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            let resource = NonFungibleVault::take(node_id, delta, api)?;

            locked = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
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
            locked.ids.entry(id.clone()).or_default().add_assign(1);
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LockedNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut locked: &mut LockedNonFungibleResource = api.kernel_get_substate_ref_mut(handle)?;

        // Take from liquid if needed
        let delta: BTreeSet<NonFungibleLocalId> = ids
            .iter()
            .cloned()
            .filter(|id| !locked.ids.contains_key(id))
            .collect();
        NonFungibleVault::take_non_fungibles(node_id, &delta, api)?;

        // Increase lock count
        locked = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
        for id in &ids {
            locked.ids.entry(id.clone()).or_default().add_assign(1);
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Vault(VaultOffset::LockedNonFungible),
            LockFlags::MUTABLE,
        )?;
        let locked: &mut LockedNonFungibleResource = api.kernel_get_substate_ref_mut(handle)?;

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
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultTakeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

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
            let bucket_id = api.new_object(
                BUCKET_BLUEPRINT,
                vec![
                    scrypto_encode(&BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    })
                    .unwrap(),
                    scrypto_encode(&taken).unwrap(),
                    scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                    scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
                    scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                ],
            )?;

            RENodeId::Object(bucket_id)
        } else {
            // Take
            let taken = NonFungibleVault::take(receiver, input.amount, api)?;

            // Create node
            let bucket_id = api.new_object(
                BUCKET_BLUEPRINT,
                vec![
                    scrypto_encode(&BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    })
                    .unwrap(),
                    scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                    scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                    scrypto_encode(&taken).unwrap(),
                    scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                ],
            )?;

            RENodeId::Object(bucket_id)
        };
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
    }

    pub fn take_non_fungibles<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultTakeNonFungiblesInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

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
            let bucket_id = api.new_object(
                BUCKET_BLUEPRINT,
                vec![
                    scrypto_encode(&BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    })
                    .unwrap(),
                    scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                    scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                    scrypto_encode(&taken).unwrap(),
                    scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                ],
            )?;

            Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
        }
    }

    pub fn put<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultPutInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        // Drop other bucket
        let other_bucket: DroppedBucket = api
            .kernel_drop_node(RENodeId::Object(input.bucket.0))?
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
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: VaultGetAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        let amount = if info.resource_type.is_fungible() {
            FungibleVault::liquid_amount(receiver, api)?
                + FungibleVault::locked_amount(receiver, api)?
        } else {
            NonFungibleVault::liquid_amount(receiver, api)?
                + NonFungibleVault::locked_amount(receiver, api)?
        };

        Ok(IndexedScryptoValue::from_typed(&amount))
    }

    pub fn get_resource_address<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: VaultGetResourceAddressInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let info = VaultInfoSubstate::of(receiver, api)?;

        Ok(IndexedScryptoValue::from_typed(&info.resource_address))
    }

    pub fn get_non_fungible_local_ids<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: VaultGetNonFungibleLocalIdsInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_type.is_fungible() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
            ));
        } else {
            let mut ids = NonFungibleVault::liquid_non_fungible_local_ids(receiver, api)?;
            ids.extend(NonFungibleVault::locked_non_fungible_local_ids(
                receiver, api,
            )?);
            Ok(IndexedScryptoValue::from_typed(&ids))
        }
    }

    pub fn lock_fee<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultLockFeeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        // Check resource address
        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_address != RADIX_TOKEN {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::LockFeeNotRadixToken),
            ));
        }
        if !info.resource_type.check_amount(input.amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Lock the substate (with special flags)
        let vault_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Vault(VaultOffset::LiquidFungible),
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        // Take by amount
        let fee = {
            let vault: &mut LiquidFungibleResource =
                api.kernel_get_substate_ref_mut(vault_handle)?;

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
            let vault: &mut LiquidFungibleResource =
                api.kernel_get_substate_ref_mut(vault_handle)?;
            vault.put(changes).expect("Failed to put fee changes");
        }

        // Emitting an event once the fee has been locked
        Runtime::emit_event(
            api,
            LockFeeEvent {
                amount: input.amount,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn recall<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultRecallInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        if !info.resource_type.check_amount(input.amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let node_id = if info.resource_type.is_fungible() {
            let taken = FungibleVault::take(receiver, input.amount, api)?;
            let bucket_id = api.new_object(
                BUCKET_BLUEPRINT,
                vec![
                    scrypto_encode(&BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    })
                    .unwrap(),
                    scrypto_encode(&taken).unwrap(),
                    scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                    scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
                    scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                ],
            )?;

            RENodeId::Object(bucket_id)
        } else {
            let taken = NonFungibleVault::take(receiver, input.amount, api)?;
            let bucket_id = api.new_object(
                BUCKET_BLUEPRINT,
                vec![
                    scrypto_encode(&BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    })
                    .unwrap(),
                    scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                    scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                    scrypto_encode(&taken).unwrap(),
                    scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                ],
            )?;

            RENodeId::Object(bucket_id)
        };
        let bucket_id = node_id.into();

        Runtime::emit_event(api, RecallResourceEvent::Amount(input.amount))?;

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
    }

    pub fn recall_non_fungibles<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultRecallNonFungiblesInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_type.is_fungible() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
            ));
        } else {
            let taken =
                NonFungibleVault::take_non_fungibles(receiver, &input.non_fungible_local_ids, api)?;

            let bucket_id = api.new_object(
                BUCKET_BLUEPRINT,
                vec![
                    scrypto_encode(&BucketInfoSubstate {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                    })
                    .unwrap(),
                    scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                    scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                    scrypto_encode(&taken).unwrap(),
                    scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                ],
            )?;

            Runtime::emit_event(api, RecallResourceEvent::Ids(input.non_fungible_local_ids))?;

            Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
        }
    }

    pub fn create_proof<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: VaultCreateProofInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        let node_id = if info.resource_type.is_fungible() {
            let amount = FungibleVault::liquid_amount(receiver, api)?
                + FungibleVault::locked_amount(receiver, api)?;

            let proof_info = ProofInfoSubstate {
                resource_address: info.resource_address,
                resource_type: info.resource_type,
                restricted: false,
            };
            let proof = FungibleVault::lock_amount(receiver, amount, api)?;

            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                vec![
                    scrypto_encode(&proof_info).unwrap(),
                    scrypto_encode(&proof).unwrap(),
                    scrypto_encode(&NonFungibleProof::default()).unwrap(),
                ],
            )?;

            RENodeId::Object(proof_id)
        } else {
            let amount = NonFungibleVault::liquid_amount(receiver, api)?
                + NonFungibleVault::locked_amount(receiver, api)?;

            let proof_info = ProofInfoSubstate {
                resource_address: info.resource_address,
                resource_type: info.resource_type,
                restricted: false,
            };
            let proof = NonFungibleVault::lock_amount(receiver, amount, api)?;

            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                vec![
                    scrypto_encode(&proof_info).unwrap(),
                    scrypto_encode(&FungibleProof::default()).unwrap(),
                    scrypto_encode(&proof).unwrap(),
                ],
            )?;

            RENodeId::Object(proof_id)
        };

        let proof_id = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub fn create_proof_by_amount<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultCreateProofByAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let info = VaultInfoSubstate::of(receiver, api)?;
        if !info.resource_type.check_amount(input.amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let node_id = if info.resource_type.is_fungible() {
            let proof_info = ProofInfoSubstate {
                resource_address: info.resource_address,
                resource_type: info.resource_type,
                restricted: false,
            };
            let proof = FungibleVault::lock_amount(receiver, input.amount, api)?;
            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                vec![
                    scrypto_encode(&proof_info).unwrap(),
                    scrypto_encode(&proof).unwrap(),
                    scrypto_encode(&NonFungibleProof::default()).unwrap(),
                ],
            )?;

            RENodeId::Object(proof_id)
        } else {
            let proof_info = ProofInfoSubstate {
                resource_address: info.resource_address,
                resource_type: info.resource_type,
                restricted: false,
            };
            let proof = NonFungibleVault::lock_amount(receiver, input.amount, api)?;
            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                vec![
                    scrypto_encode(&proof_info).unwrap(),
                    scrypto_encode(&FungibleProof::default()).unwrap(),
                    scrypto_encode(&proof).unwrap(),
                ],
            )?;

            RENodeId::Object(proof_id)
        };

        let proof_id = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub fn create_proof_by_ids<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultCreateProofByIdsInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let info = VaultInfoSubstate::of(receiver, api)?;

        if info.resource_type.is_fungible() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NonFungibleOperationNotSupported),
            ));
        } else {
            let proof_info = ProofInfoSubstate {
                resource_address: info.resource_address,
                resource_type: info.resource_type,
                restricted: false,
            };
            let proof = NonFungibleVault::lock_non_fungibles(receiver, input.ids, api)?;
            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                vec![
                    scrypto_encode(&proof_info).unwrap(),
                    scrypto_encode(&FungibleProof::default()).unwrap(),
                    scrypto_encode(&proof).unwrap(),
                ],
            )?;
            Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
        }
    }

    //===================
    // Protected method
    //===================

    // FIXME: set up auth

    pub fn lock_amount<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultLockAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        FungibleVault::lock_amount(receiver, input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn lock_non_fungibles<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultLockNonFungiblesInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        NonFungibleVault::lock_non_fungibles(receiver, input.local_ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn unlock_amount<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultUnlockAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        FungibleVault::unlock_amount(receiver, input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn unlock_non_fungibles<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: VaultUnlockNonFungiblesInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        NonFungibleVault::unlock_non_fungibles(receiver, input.local_ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
