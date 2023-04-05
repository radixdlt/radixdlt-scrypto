use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::heap::{DroppedBucket, DroppedBucketResource};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct FungibleVaultDivisibilitySubstate {
    pub divisibility: u8,
}

pub struct FungibleVaultBlueprint;

impl FungibleVaultBlueprint {
    fn check_amount(amount: &Decimal, divisibility: u8) -> bool {
        !amount.is_negative()
            && amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into()))
                == BnumI256::from(0)
    }

    fn get_divisibility<Y>(receiver: &NodeId, api: &mut Y) -> Result<u8, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle =
            api.sys_lock_substate(receiver, &VaultOffset::Info.into(), LockFlags::read_only())?;
        let info: FungibleVaultDivisibilitySubstate = api.sys_read_substate_typed(handle)?;
        let divisibility = info.divisibility;
        api.sys_drop_lock(handle)?;
        Ok(divisibility)
    }

    pub fn take<Y>(receiver: &NodeId, amount: &Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let divisibility = Self::get_divisibility(receiver, api)?;
        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );

        // Check amount
        if !Self::check_amount(amount, divisibility) {
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
                    resource_address,
                    resource_type: ResourceType::Fungible { divisibility },
                })
                .unwrap(),
                scrypto_encode(&taken).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub fn put<Y>(receiver: &NodeId, bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Drop other bucket
        let other_bucket: DroppedBucket = api.kernel_drop_node(bucket.0.as_node_id())?.into();

        // Check resource address
        {
            let resource_address = ResourceAddress::new_unchecked(
                api.get_object_info(receiver)?.type_parent.unwrap().into(),
            );
            if resource_address != other_bucket.info.resource_address {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::VaultError(VaultError::MismatchingResource),
                ));
            }
        }

        // Put
        if let DroppedBucketResource::Fungible(r) = other_bucket.resource {
            FungibleVault::put(receiver, r, api)?;
        } else {
            panic!("expecting fungible bucket")
        }

        Ok(())
    }

    pub fn get_amount<Y>(receiver: &NodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let amount = FungibleVault::liquid_amount(receiver, api)?
            + FungibleVault::locked_amount(receiver, api)?;

        Ok(amount)
    }

    pub fn lock_fee<Y>(
        receiver: &NodeId,
        amount: Decimal,
        contingent: bool,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Check resource address and amount
        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        if resource_address != RADIX_TOKEN {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::LockFeeNotRadixToken),
            ));
        }

        let divisibility = Self::get_divisibility(receiver, api)?;
        if !Self::check_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Lock the substate (with special flags)
        let vault_handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LiquidFungible.into(),
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        // Take fee from the vault
        let mut vault: LiquidFungibleResource = api.sys_read_substate_typed(vault_handle)?;
        let fee = vault.take_by_amount(amount).map_err(|_| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::LockFeeInsufficientBalance,
            ))
        })?;

        // Credit cost units
        let changes = api.credit_cost_units(receiver.clone().into(), fee, contingent)?;

        // Keep changes
        if !changes.is_empty() {
            vault.put(changes).expect("Failed to put fee changes");
        }

        // Flush updates
        api.sys_write_substate_typed(vault_handle, &vault)?;
        api.sys_drop_lock(vault_handle)?;

        // Emitting an event once the fee has been locked
        Runtime::emit_event(api, LockFeeEvent { amount })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn recall<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let divisibility = Self::get_divisibility(receiver, api)?;
        if !Self::check_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        let taken = FungibleVault::take(receiver, amount, api)?;
        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&BucketInfoSubstate {
                    resource_address,
                    resource_type: ResourceType::Fungible { divisibility },
                })
                .unwrap(),
                scrypto_encode(&taken).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Runtime::emit_event(api, RecallResourceEvent::Amount(amount))?;
        Ok(Bucket(Own(bucket_id)))
    }

    pub fn create_proof<Y>(receiver: &NodeId, api: &mut Y) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let amount = FungibleVault::liquid_amount(receiver, api)?
            + FungibleVault::locked_amount(receiver, api)?;

        let divisibility = Self::get_divisibility(receiver, api)?;
        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        let proof_info = ProofInfoSubstate {
            resource_address,
            resource_type: ResourceType::Fungible { divisibility },
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

        Ok(Proof(Own(proof_id)))
    }

    pub fn create_proof_by_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let divisibility = Self::get_divisibility(receiver, api)?;
        if !Self::check_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        let proof_info = ProofInfoSubstate {
            resource_address,
            resource_type: ResourceType::Fungible { divisibility },
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

        Ok(Proof(Own(proof_id)))
    }

    //===================
    // Protected method
    //===================

    // FIXME: set up auth

    pub fn lock_amount<Y>(
        receiver: &NodeId,
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
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        FungibleVault::unlock_amount(receiver, amount, api)?;

        Ok(())
    }
}

pub struct FungibleVault;

impl FungibleVault {
    pub fn liquid_amount<Y>(receiver: &NodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LiquidFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidFungibleResource = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn locked_amount<Y>(receiver: &NodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LockedFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedFungibleResource = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn is_locked<Y>(receiver: &NodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        Ok(!Self::locked_amount(receiver, api)?.is_zero())
    }

    pub fn take<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LiquidFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidFungibleResource = api.sys_read_substate_typed(handle)?;
        let taken = substate_ref.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.sys_write_substate_typed(handle, &substate_ref)?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, WithdrawResourceEvent::Amount(amount))?;

        Ok(taken)
    }

    pub fn put<Y>(
        receiver: &NodeId,
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
            receiver,
            &VaultOffset::LiquidFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidFungibleResource = api.sys_read_substate_typed(handle)?;
        substate_ref.put(resource).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.sys_write_substate_typed(handle, &substate_ref)?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(())
    }

    // protected method
    pub fn lock_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<FungibleProof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LockedFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedFungibleResource = api.sys_read_substate_typed(handle)?;
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            FungibleVault::take(receiver, delta, api)?;
        }

        // Increase lock count
        locked.amounts.entry(amount).or_default().add_assign(1);
        api.sys_write_substate_typed(handle, &locked)?;

        // Issue proof
        Ok(FungibleProof::new(
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
    pub fn unlock_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LockedFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedFungibleResource = api.sys_read_substate_typed(handle)?;

        let max_locked = locked.amount();
        let cnt = locked
            .amounts
            .remove(&amount)
            .expect("Attempted to unlock an amount that is not locked");
        if cnt > 1 {
            locked.amounts.insert(amount, cnt - 1);
        }

        api.sys_write_substate_typed(handle, &locked)?;

        let delta = max_locked - locked.amount();
        FungibleVault::put(receiver, LiquidFungibleResource::new(delta), api)
    }
}
