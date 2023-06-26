use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::resource::NativeBucket;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_OUTER_OBJECT, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;

pub use radix_engine_interface::blueprints::resource::LiquidFungibleResource as FungibleVaultBalanceSubstate;

pub struct FungibleVaultBlueprint;

impl FungibleVaultBlueprint {
    fn get_divisibility<Y>(api: &mut Y) -> Result<u8, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_OUTER_OBJECT,
            FungibleResourceManagerField::Divisibility.into(),
            LockFlags::read_only(),
        )?;
        let divisibility: u8 = api.field_lock_read_typed(handle)?;
        api.field_lock_release(handle)?;
        Ok(divisibility)
    }

    pub fn take<Y>(amount: &Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::WITHDRAW, api)?;

        let divisibility = Self::get_divisibility(api)?;

        // Check amount
        if !check_fungible_amount(amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Take
        let taken = FungibleVault::take(*amount, api)?;

        // Create node
        FungibleResourceManagerBlueprint::create_bucket(taken.amount(), api)
    }

    pub fn put<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::DEPOSIT, api)?;

        // Drop other bucket
        let other_bucket = drop_fungible_bucket(bucket.0.as_node_id(), api)?;

        // Put
        FungibleVault::put(other_bucket.liquid, api)?;

        Ok(())
    }

    pub fn get_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let amount = FungibleVault::liquid_amount(api)? + FungibleVault::locked_amount(api)?;

        Ok(amount)
    }

    pub fn lock_fee<Y>(
        receiver: &NodeId,
        amount: Decimal,
        contingent: bool,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::WITHDRAW, api)?;

        // Check resource address and amount
        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_info()?.get_outer_object().into());
        if resource_address != RADIX_TOKEN {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::LockFeeNotRadixToken),
            ));
        }

        let divisibility = Self::get_divisibility(api)?;
        if !check_fungible_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Lock the substate (with special flags)
        let vault_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleVaultField::LiquidFungible.into(),
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        // Take fee from the vault
        let mut vault: LiquidFungibleResource = api.field_lock_read_typed(vault_handle)?;
        let fee = vault.take_by_amount(amount).map_err(|_| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::LockFeeInsufficientBalance,
            ))
        })?;

        // Credit cost units
        let changes = api.credit_cost_units(receiver.clone().into(), fee, contingent)?;

        // Keep changes
        if !changes.is_empty() {
            vault.put(changes);
        }

        // Flush updates
        api.field_lock_write_typed(vault_handle, &vault)?;
        api.field_lock_release(vault_handle)?;

        // Emitting an event once the fee has been locked
        Runtime::emit_event(api, LockFeeEvent { amount })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn recall<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_recallable(api)?;

        let divisibility = Self::get_divisibility(api)?;
        if !check_fungible_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let taken = FungibleVault::take(amount, api)?;

        let bucket = FungibleResourceManagerBlueprint::create_bucket(taken.amount(), api)?;

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
            FungibleVaultField::VaultFrozenFlag.into(),
            LockFlags::MUTABLE,
        )?;

        let mut frozen: VaultFrozenFlag = api.field_lock_read_typed(frozen_flag_handle)?;
        frozen.frozen.insert(to_freeze);
        api.field_lock_write_typed(frozen_flag_handle, &frozen)?;

        Ok(())
    }

    pub fn unfreeze<Y>(to_unfreeze: VaultFreezeFlags, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_freezable(api)?;

        let frozen_flag_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleVaultField::VaultFrozenFlag.into(),
            LockFlags::MUTABLE,
        )?;
        let mut frozen: VaultFrozenFlag = api.field_lock_read_typed(frozen_flag_handle)?;
        frozen.frozen.remove(to_unfreeze);
        api.field_lock_write_typed(frozen_flag_handle, &frozen)?;

        Ok(())
    }

    pub fn create_proof<Y>(receiver: &NodeId, api: &mut Y) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::create_proof_of_amount(receiver, Decimal::ONE, api)
    }

    pub fn create_proof_of_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let divisibility = Self::get_divisibility(api)?;
        if !check_fungible_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let proof_info = ProofMoveableSubstate { restricted: false };
        let proof = FungibleVault::lock_amount(receiver, amount, api)?;
        let proof_id = api.new_simple_object(
            FUNGIBLE_PROOF_BLUEPRINT,
            vec![
                scrypto_encode(&proof_info).unwrap(),
                scrypto_encode(&proof).unwrap(),
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

    //===================
    // Protected method
    //===================

    pub fn lock_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        FungibleVault::lock_amount(receiver, amount, api)?;
        Ok(())
    }

    pub fn unlock_amount<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        FungibleVault::unlock_amount(amount, api)?;

        Ok(())
    }

    fn assert_not_frozen<Y>(flags: VaultFreezeFlags, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(OBJECT_HANDLE_OUTER_OBJECT, VAULT_FREEZE_FEATURE)? {
            return Ok(());
        }

        let frozen_flag_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleVaultField::VaultFrozenFlag.into(),
            LockFlags::MUTABLE,
        )?;
        let frozen: VaultFrozenFlag = api.field_lock_read_typed(frozen_flag_handle)?;

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
}

pub struct FungibleVault;

impl FungibleVault {
    pub fn liquid_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleVaultField::LiquidFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidFungibleResource = api.field_lock_read_typed(handle)?;
        let amount = substate_ref.amount();
        api.field_lock_release(handle)?;
        Ok(amount)
    }

    pub fn locked_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleVaultField::LockedFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedFungibleResource = api.field_lock_read_typed(handle)?;
        let amount = substate_ref.amount();
        api.field_lock_release(handle)?;
        Ok(amount)
    }

    pub fn take<Y>(amount: Decimal, api: &mut Y) -> Result<LiquidFungibleResource, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleVaultField::LiquidFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidFungibleResource = api.field_lock_read_typed(handle)?;
        let taken = substate_ref.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.field_lock_write_typed(handle, &substate_ref)?;
        api.field_lock_release(handle)?;

        Runtime::emit_event(api, WithdrawResourceEvent::Amount(amount))?;

        Ok(taken)
    }

    pub fn put<Y>(resource: LiquidFungibleResource, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let event = DepositResourceEvent::Amount(resource.amount());

        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleVaultField::LiquidFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidFungibleResource = api.field_lock_read_typed(handle)?;
        substate_ref.put(resource);
        api.field_lock_write_typed(handle, &substate_ref)?;
        api.field_lock_release(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(())
    }

    // protected method
    pub fn lock_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<FungibleProofSubstate, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleVaultField::LockedFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedFungibleResource = api.field_lock_read_typed(handle)?;
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            FungibleVault::take(delta, api)?;
        }

        // Increase lock count
        locked.amounts.entry(amount).or_default().add_assign(1);
        api.field_lock_write_typed(handle, &locked)?;

        // Issue proof
        Ok(FungibleProofSubstate::new(
            amount,
            btreemap!(
                LocalRef::Vault(Reference(receiver.clone().into())) => amount
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
        })?)
    }

    // protected method
    pub fn unlock_amount<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleVaultField::LockedFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedFungibleResource = api.field_lock_read_typed(handle)?;

        let max_locked = locked.amount();
        let cnt = locked
            .amounts
            .remove(&amount)
            .expect("Attempted to unlock an amount that is not locked");
        if cnt > 1 {
            locked.amounts.insert(amount, cnt - 1);
        }

        api.field_lock_write_typed(handle, &locked)?;

        let delta = max_locked - locked.amount();
        FungibleVault::put(LiquidFungibleResource::new(delta), api)
    }
}
