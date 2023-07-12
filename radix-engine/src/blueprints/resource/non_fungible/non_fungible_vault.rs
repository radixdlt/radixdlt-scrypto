use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::resource::NativeBucket;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::{
    ClientApi, CollectionIndex, FieldValue, LockFlags, OBJECT_HANDLE_OUTER_OBJECT,
    OBJECT_HANDLE_SELF,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NonFungibleVaultError {
    MissingId(NonFungibleLocalId),
    NotEnoughAmount,
}

pub use radix_engine_interface::blueprints::resource::LiquidNonFungibleVault as NonFungibleVaultBalanceSubstate;

pub const NON_FUNGIBLE_VAULT_CONTENTS_INDEX: CollectionIndex = 0u8;

pub use crate::types::NonFungibleLocalId as NonFungibleVaultContentsEntry;

pub struct NonFungibleVaultBlueprint;

impl NonFungibleVaultBlueprint {
    pub fn take<Y>(amount: &Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::take_advanced(amount, WithdrawStrategy::Exact, api)
    }

    pub fn take_advanced<Y>(
        amount: &Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::WITHDRAW, api)?;

        let amount = amount.for_withdrawal(0, withdraw_strategy);

        // Check amount
        let n = check_non_fungible_amount(&amount).map_err(|_| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::InvalidAmount))
        })?;

        // Take
        let taken = Self::internal_take_by_amount(n, api)?;

        // Create node
        NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)
    }

    pub fn take_non_fungibles<Y>(
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::WITHDRAW, api)?;

        // Take
        let taken = Self::internal_take_non_fungibles(non_fungible_local_ids, api)?;

        // Create node
        NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)
    }

    pub fn put<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::DEPOSIT, api)?;

        // Drop other bucket
        let other_bucket = drop_non_fungible_bucket(bucket.0.as_node_id(), api)?;

        // Put
        Self::internal_put(other_bucket.liquid, api)?;

        Ok(())
    }

    pub fn get_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let amount = Self::liquid_amount(api)? + Self::locked_amount(api)?;

        Ok(amount)
    }

    pub fn get_non_fungible_local_ids<Y>(
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut ids = Self::liquid_non_fungible_local_ids(api)?;
        ids.extend(Self::locked_non_fungible_local_ids(api)?);
        Ok(ids)
    }

    pub fn recall<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_recallable(api)?;

        let n = check_non_fungible_amount(&amount).map_err(|_| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::InvalidAmount))
        })?;

        let taken = Self::internal_take_by_amount(n, api)?;

        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)?;

        Runtime::emit_event(api, RecallResourceEvent::Amount(amount))?;

        Ok(bucket)
    }

    pub fn freeze<Y>(to_freeze: VaultFreezeFlags, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_freezable(api)?;

        let frozen_flag_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::VaultFrozenFlag.into(),
            LockFlags::MUTABLE,
        )?;

        let mut frozen: VaultFrozenFlag = api.field_read_typed(frozen_flag_handle)?;
        frozen.frozen.insert(to_freeze);
        api.field_write_typed(frozen_flag_handle, &frozen)?;

        Ok(())
    }

    pub fn unfreeze<Y>(to_unfreeze: VaultFreezeFlags, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_freezable(api)?;

        let frozen_flag_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::VaultFrozenFlag.into(),
            LockFlags::MUTABLE,
        )?;
        let mut frozen: VaultFrozenFlag = api.field_read_typed(frozen_flag_handle)?;
        frozen.frozen.remove(to_unfreeze);
        api.field_write_typed(frozen_flag_handle, &frozen)?;

        Ok(())
    }

    pub fn recall_non_fungibles<Y>(
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_recallable(api)?;

        let taken = Self::internal_take_non_fungibles(&non_fungible_local_ids, api)?;

        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)?;

        Runtime::emit_event(api, RecallResourceEvent::Ids(non_fungible_local_ids))?;

        Ok(bucket)
    }

    pub fn create_proof_of_non_fungibles<Y>(
        receiver: &NodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::lock_non_fungibles(&ids, api)?;

        let proof_info = ProofMoveableSubstate { restricted: false };
        let proof_evidence = NonFungibleProofSubstate::new(
            ids.clone(),
            btreemap!(
                LocalRef::Vault(Reference(receiver.clone().into()))=> ids
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
        })?;
        let proof_id = api.new_simple_object(
            NON_FUNGIBLE_PROOF_BLUEPRINT,
            vec![
                FieldValue::new(&proof_info),
                FieldValue::new(&proof_evidence),
            ],
        )?;
        Ok(Proof(Own(proof_id)))
    }

    pub fn burn<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::BURN, api)?;

        Self::take(&amount, api)?.package_burn(api)?;
        Ok(())
    }

    pub fn burn_non_fungibles<Y>(
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::BURN, api)?;

        Self::take_non_fungibles(non_fungible_local_ids, api)?.package_burn(api)?;
        Ok(())
    }

    //===================
    // Protected methods
    //===================

    pub fn lock_non_fungibles<Y>(
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::LockedNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.field_read_typed(handle)?;

        // Take from liquid if needed
        let delta: BTreeSet<NonFungibleLocalId> = ids
            .iter()
            .cloned()
            .filter(|id| !locked.ids.contains_key(id))
            .collect();
        Self::internal_take_non_fungibles(&delta, api)?;

        // Increase lock count
        for id in ids {
            locked.ids.entry(id.clone()).or_default().add_assign(1);
        }

        api.field_write_typed(handle, &locked)?;

        // Issue proof
        Ok(())
    }

    pub fn unlock_non_fungibles<Y>(
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::LockedNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.field_read_typed(handle)?;

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

        api.field_write_typed(handle, &locked)?;

        Self::internal_put(LiquidNonFungibleResource::new(liquid_non_fungibles), api)
    }

    //===================
    // Helper methods
    //===================

    fn assert_not_frozen<Y>(flags: VaultFreezeFlags, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(OBJECT_HANDLE_OUTER_OBJECT, VAULT_FREEZE_FEATURE)? {
            return Ok(());
        }

        let frozen_flag_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::VaultFrozenFlag.into(),
            LockFlags::read_only(),
        )?;
        let frozen: VaultFrozenFlag = api.field_read_typed(frozen_flag_handle)?;
        api.field_close(frozen_flag_handle)?;

        if frozen.frozen.intersects(flags) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::VaultIsFrozen),
            ));
        }

        Ok(())
    }

    fn assert_freezable<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(OBJECT_HANDLE_OUTER_OBJECT, VAULT_FREEZE_FEATURE)? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NotFreezable),
            ));
        }

        Ok(())
    }

    fn assert_recallable<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(OBJECT_HANDLE_OUTER_OBJECT, VAULT_RECALL_FEATURE)? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NotRecallable),
            ));
        }

        Ok(())
    }

    fn liquid_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::LiquidNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidNonFungibleVault = api.field_read_typed(handle)?;
        let amount = substate_ref.amount;
        api.field_close(handle)?;
        Ok(amount)
    }

    fn locked_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::LockedNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedNonFungibleResource = api.field_read_typed(handle)?;
        let amount = substate_ref.amount();
        api.field_close(handle)?;
        Ok(amount)
    }

    fn liquid_non_fungible_local_ids<Y>(
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // FIXME: only allow a certain amount to be returned
        let items: Vec<(NonFungibleLocalId, NonFungibleLocalId)> = api.actor_index_scan_typed(
            OBJECT_HANDLE_SELF,
            NON_FUNGIBLE_VAULT_CONTENTS_INDEX,
            u32::MAX,
        )?;
        let ids = items.into_iter().map(|(k, _v)| k).collect();
        Ok(ids)
    }

    fn locked_non_fungible_local_ids<Y>(
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::LockedNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedNonFungibleResource = api.field_read_typed(handle)?;
        let ids = substate_ref.ids();
        api.field_close(handle)?;
        Ok(ids)
    }

    fn internal_take_by_amount<Y>(
        n: u32,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // deduct from liquidity pool
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::LiquidNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidNonFungibleVault = api.field_read_typed(handle)?;

        if substate_ref.amount < Decimal::from(n) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleVaultError(NonFungibleVaultError::NotEnoughAmount),
            ));
        }
        substate_ref.amount -= Decimal::from(n);

        let taken = {
            let ids: Vec<(NonFungibleLocalId, NonFungibleLocalId)> = api.actor_index_take_typed(
                OBJECT_HANDLE_SELF,
                NON_FUNGIBLE_VAULT_CONTENTS_INDEX,
                n,
            )?;
            LiquidNonFungibleResource {
                ids: ids.into_iter().map(|(key, _value)| key).collect(),
            }
        };

        api.field_write_typed(handle, &substate_ref)?;
        api.field_close(handle)?;

        Runtime::emit_event(api, WithdrawResourceEvent::Ids(taken.ids.clone()))?;

        Ok(taken)
    }

    pub fn internal_take_non_fungibles<Y>(
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::LiquidNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidNonFungibleVault = api.field_read_typed(handle)?;

        substate_ref.amount -= Decimal::from(ids.len());

        // TODO: Batch remove
        for id in ids {
            let removed = api.actor_index_remove(
                OBJECT_HANDLE_SELF,
                NON_FUNGIBLE_VAULT_CONTENTS_INDEX,
                scrypto_encode(id).unwrap(),
            )?;

            if removed.is_none() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleVaultError(NonFungibleVaultError::MissingId(
                        id.clone(),
                    )),
                ));
            }
        }

        Runtime::emit_event(api, WithdrawResourceEvent::Ids(ids.clone()))?;
        api.field_write_typed(handle, &substate_ref)?;
        api.field_close(handle)?;

        Ok(LiquidNonFungibleResource::new(ids.clone()))
    }

    pub fn internal_put<Y>(
        resource: LiquidNonFungibleResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let event = DepositResourceEvent::Ids(resource.ids().clone());

        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleVaultField::LiquidNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut vault: LiquidNonFungibleVault = api.field_read_typed(handle)?;

        vault.amount += Decimal::from(resource.ids.len());

        // update liquidity
        // TODO: Batch update
        // TODO: Rather than insert, use create_unique?
        for id in resource.ids {
            api.actor_index_insert_typed(
                OBJECT_HANDLE_SELF,
                NON_FUNGIBLE_VAULT_CONTENTS_INDEX,
                scrypto_encode(&id).unwrap(),
                id,
            )?;
        }

        api.field_write_typed(handle, &vault)?;
        api.field_close(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(())
    }
}
