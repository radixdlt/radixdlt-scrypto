use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::ApplicationError;
use crate::kernel::heap::{DroppedBucket, DroppedBucketResource};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::*;

pub struct FungibleVaultBlueprint;

impl FungibleVaultBlueprint {
    pub fn take<Y>(
        receiver: &RENodeId,
        amount: &Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Check amount
        let info = VaultInfoSubstate::of(receiver, api)?;
        if !info.resource_type.check_amount(*amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Take
        let taken = FungibleVault::take(receiver, *amount, api)?;

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

        Ok(Bucket(bucket_id))
    }

    pub fn put<Y>(receiver: &RENodeId, bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Drop other bucket
        let other_bucket: DroppedBucket = api.kernel_drop_node(&RENodeId::Object(bucket.0))?.into();

        // Check resource address
        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_address != other_bucket.info.resource_address {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::MismatchingResource),
            ));
        }

        // Put
        if let DroppedBucketResource::Fungible(r) = other_bucket.resource {
            FungibleVault::put(receiver, r, api)?;
        } else {
            panic!("expecting fungible bucket")
        }

        Ok(())
    }

    pub fn get_amount<Y>(receiver: &RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let amount = FungibleVault::liquid_amount(receiver, api)?
            + FungibleVault::locked_amount(receiver, api)?;

        Ok(amount)
    }

    pub fn lock_fee<Y>(
        receiver: &RENodeId,
        amount: Decimal,
        contingent: bool,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Check resource address
        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_address != RADIX_TOKEN {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::LockFeeNotRadixToken),
            ));
        }
        if !info.resource_type.check_amount(amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Lock the substate (with special flags)
        let vault_handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::Vault(VaultOffset::LiquidFungible),
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        // Take by amount
        let fee = {
            let vault: &mut LiquidFungibleResource =
                api.kernel_get_substate_ref_mut(vault_handle)?;

            // Take fee from the vault
            vault.take_by_amount(amount).map_err(|_| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::LockFeeInsufficientBalance,
                ))
            })?
        };

        // Credit cost units
        let changes = api.credit_cost_units(receiver.clone().into(), fee, contingent)?;

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
                amount,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn recall<Y>(
        receiver: &RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let info = VaultInfoSubstate::of(receiver, api)?;
        if !info.resource_type.check_amount(amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let taken = FungibleVault::take(receiver, amount, api)?;
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

        Runtime::emit_event(api, RecallResourceEvent::Amount(amount))?;
        Ok(Bucket(bucket_id))
    }

    pub fn create_proof<Y>(
        receiver: &RENodeId,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let info = VaultInfoSubstate::of(receiver, api)?;
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

        Ok(Proof(proof_id))
    }

    pub fn create_proof_by_amount<Y>(
        receiver: &RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let info = VaultInfoSubstate::of(receiver, api)?;
        if !info.resource_type.check_amount(amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

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

        Ok(Proof(proof_id))
    }

    //===================
    // Protected method
    //===================

    // FIXME: set up auth

    pub fn lock_amount<Y>(
        receiver: &RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        FungibleVault::lock_amount(receiver, amount, api)?;
        Ok(())
    }

    pub fn unlock_amount<Y>(
        receiver: &RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        FungibleVault::unlock_amount(receiver, amount, api)?;

        Ok(())
    }

    pub fn get_resource_address<Y>(
        receiver: &RENodeId,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let info = VaultInfoSubstate::of(receiver, api)?;
        Ok(info.resource_address)
    }
}

pub struct FungibleVault;

impl FungibleVault {
    pub fn liquid_amount<Y>(receiver: &RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::Vault(VaultOffset::LiquidFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref: &LiquidFungibleResource = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn locked_amount<Y>(receiver: &RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::Vault(VaultOffset::LockedFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref: &LockedFungibleResource = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn is_locked<Y>(receiver: &RENodeId, api: &mut Y) -> Result<bool, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        Ok(!Self::locked_amount(receiver, api)?.is_zero())
    }

    pub fn take<Y>(
        receiver: &RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidFungibleResource, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver.clone(),
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
        receiver: &RENodeId,
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
            receiver.clone(),
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
        receiver: &RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<FungibleProof, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::Vault(VaultOffset::LockedFungible),
            LockFlags::MUTABLE,
        )?;
        let mut locked: &mut LockedFungibleResource = api.kernel_get_substate_ref_mut(handle)?;
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            FungibleVault::take(receiver, delta, api)?;
        }

        // Increase lock count
        locked = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
        locked.amounts.entry(amount).or_default().add_assign(1);

        // Issue proof
        Ok(FungibleProof::new(
            amount,
            btreemap!(
                LocalRef::Vault(receiver.clone().into()) => amount
            ),
        )
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
            })?)
    }

    // protected method
    pub fn unlock_amount<Y>(
        receiver: &RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver.clone(),
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
        FungibleVault::put(receiver, LiquidFungibleResource::new(delta), api)
    }
}