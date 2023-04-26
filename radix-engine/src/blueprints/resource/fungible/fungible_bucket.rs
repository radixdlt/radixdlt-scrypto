use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, SystemUpstreamError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use radix_engine_interface::api::{ClientApi, LockFlags};
use radix_engine_interface::blueprints::resource::*;

pub struct FungibleBucket;

impl FungibleBucket {
    pub fn liquid_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(FungibleBucketOffset::Liquid.into(), LockFlags::read_only())?;
        let substate_ref: LiquidFungibleResource = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn locked_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(FungibleBucketOffset::Locked.into(), LockFlags::read_only())?;
        let substate_ref: LockedFungibleResource = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn take<Y>(amount: Decimal, api: &mut Y) -> Result<LiquidFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(FungibleBucketOffset::Liquid.into(), LockFlags::MUTABLE)?;
        let mut substate: LiquidFungibleResource = api.sys_read_substate_typed(handle)?;
        let taken = substate.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;
        api.sys_write_substate_typed(handle, &substate)?;
        api.sys_drop_lock(handle)?;
        Ok(taken)
    }

    pub fn put<Y>(resource: LiquidFungibleResource, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let handle = api.lock_field(FungibleBucketOffset::Liquid.into(), LockFlags::MUTABLE)?;
        let mut substate: LiquidFungibleResource = api.sys_read_substate_typed(handle)?;
        substate.put(resource).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;
        api.sys_write_substate_typed(handle, &substate)?;
        api.sys_drop_lock(handle)?;
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
        let handle = api.lock_field(FungibleBucketOffset::Locked.into(), LockFlags::MUTABLE)?;
        let mut locked: LockedFungibleResource = api.sys_read_substate_typed(handle)?;
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            FungibleBucket::take(delta, api)?;
        }

        // Increase lock count
        locked.amounts.entry(amount).or_default().add_assign(1);

        api.sys_write_substate_typed(handle, &locked)?;

        // Issue proof
        Ok(FungibleProof::new(
            amount,
            btreemap!(
                LocalRef::Bucket(Reference(receiver.clone())) => amount
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?)
    }

    // protected method
    pub fn unlock_amount<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(FungibleBucketOffset::Locked.into(), LockFlags::MUTABLE)?;
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
        FungibleBucket::put(LiquidFungibleResource::new(delta), api)
    }
}

pub struct FungibleBucketBlueprint;

impl FungibleBucketBlueprint {
    fn get_divisibility<Y>(api: &mut Y) -> Result<u8, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let divisibility_handle = api.lock_parent_field(
            FungibleResourceManagerOffset::Divisibility.into(),
            LockFlags::read_only(),
        )?;
        let divisibility: u8 = api.sys_read_substate_typed(divisibility_handle)?;
        api.sys_drop_lock(divisibility_handle)?;
        Ok(divisibility)
    }

    pub fn take<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: BucketTakeInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        // Check amount
        {
            let divisibility = Self::get_divisibility(api)?;
            if !(check_amount(Some(divisibility), input.amount)) {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::BucketError(BucketError::InvalidAmount),
                ));
            }
        }

        // Take
        let taken = FungibleBucket::take(input.amount, api)?;

        // Create node
        let bucket = FungibleResourceManagerBlueprint::create_bucket(taken.amount(), api)?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    pub fn put<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: BucketPutInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        // Drop other bucket
        let other_bucket =
            drop_fungible_bucket(input.bucket.0.as_node_id(), api)?;

        // Put
        let rtn = FungibleBucket::put(other_bucket.liquid, api)?;

        Ok(IndexedScryptoValue::from_typed(&rtn))
    }

    pub fn get_amount<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: BucketGetAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let amount = FungibleBucket::liquid_amount(api)? + FungibleBucket::locked_amount(api)?;

        Ok(IndexedScryptoValue::from_typed(&amount))
    }

    pub fn get_resource_address<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: BucketGetResourceAddressInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let resource_address =
            ResourceAddress::new_unchecked(api.get_info()?.blueprint_parent.unwrap().into());

        Ok(IndexedScryptoValue::from_typed(&resource_address))
    }

    pub fn create_proof<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: BucketCreateProofInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let resource_address =
            ResourceAddress::new_unchecked(api.get_info()?.blueprint_parent.unwrap().into());

        let node_id = {
            let divisibility = Self::get_divisibility(api)?;
            let amount = FungibleBucket::locked_amount(api)? + FungibleBucket::liquid_amount(api)?;

            let proof_info = ProofInfoSubstate {
                resource_address,
                resource_type: ResourceType::Fungible { divisibility },
                restricted: false,
            };
            let proof = FungibleBucket::lock_amount(receiver, amount, api)?;

            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                vec![
                    scrypto_encode(&proof_info).unwrap(),
                    scrypto_encode(&proof).unwrap(),
                    scrypto_encode(&NonFungibleProof::default()).unwrap(),
                ],
            )?;
            proof_id
        };

        Ok(IndexedScryptoValue::from_typed(&Proof(Own(node_id))))
    }

    //===================
    // Protected method
    //===================

    pub fn lock_amount<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: FungibleBucketLockAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        FungibleBucket::lock_amount(receiver, input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn unlock_amount<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: FungibleBucketUnlockAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        FungibleBucket::unlock_amount(input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
