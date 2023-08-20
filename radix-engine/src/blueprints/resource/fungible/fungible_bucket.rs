use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use radix_engine_interface::api::{ClientApi, FieldValue, LockFlags, ACTOR_STATE_OUTER_OBJECT, ACTOR_STATE_SELF, ACTOR_REF_OUTER};
use radix_engine_interface::blueprints::resource::*;

pub struct FungibleBucket;

pub struct FungibleBucketBlueprint;

impl FungibleBucketBlueprint {
    fn get_divisibility<Y>(api: &mut Y) -> Result<u8, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let divisibility_handle = api.actor_open_field(
            ACTOR_STATE_OUTER_OBJECT,
            FungibleResourceManagerField::Divisibility.into(),
            LockFlags::read_only(),
        )?;
        let divisibility: u8 = api.field_read_typed(divisibility_handle)?;
        api.field_close(divisibility_handle)?;
        Ok(divisibility)
    }

    pub fn take<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::take_advanced(amount, WithdrawStrategy::Exact, api)
    }

    pub fn take_advanced<Y>(
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        // Apply withdraw strategy
        let divisibility = Self::get_divisibility(api)?;
        let amount = amount.for_withdrawal(divisibility, withdraw_strategy);

        // Check amount
        if !(check_fungible_amount(&amount, divisibility)) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::InvalidAmount),
            ));
        }

        // Take
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Liquid.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: LiquidFungibleResource = api.field_read_typed(handle)?;
        let taken = substate.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;
        api.field_write_typed(handle, &substate)?;
        api.field_close(handle)?;

        // Create node
        let bucket = FungibleResourceManagerBlueprint::create_bucket(taken.amount(), api)?;

        Ok(bucket)
    }

    pub fn put<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Drop other bucket
        let other_bucket = drop_fungible_bucket(bucket.0.as_node_id(), api)?;
        let resource = other_bucket.liquid;

        // Put
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Liquid.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: LiquidFungibleResource = api.field_read_typed(handle)?;
        substate.put(resource);
        api.field_write_typed(handle, &substate)?;
        api.field_close(handle)?;

        Ok(())
    }

    pub fn get_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Ok(Self::liquid_amount(api)?
            .safe_add(Self::locked_amount(api)?)
            .unwrap())
    }

    pub fn get_resource_address<Y>(api: &mut Y) -> Result<ResourceAddress, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_OUTER)?.into());

        Ok(resource_address)
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
                ApplicationError::BucketError(BucketError::InvalidAmount),
            ));
        }

        Self::lock_amount(amount, api)?;

        let proof_info = ProofMoveableSubstate { restricted: false };
        let proof_evidence = FungibleProofSubstate::new(
            amount,
            btreemap!(
                LocalRef::Bucket(Reference(receiver.clone())) => amount
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?;
        let proof_id = api.new_simple_object(
            FUNGIBLE_PROOF_BLUEPRINT,
            vec![
                FieldValue::new(&proof_info),
                FieldValue::new(&proof_evidence),
            ],
        )?;

        Ok(Proof(Own(proof_id)))
    }

    pub fn create_proof_of_all<Y>(receiver: &NodeId, api: &mut Y) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::create_proof_of_amount(receiver, Self::get_amount(api)?, api)
    }

    //===================
    // Protected method
    //===================

    pub fn lock_amount<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Locked.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedFungibleResource = api.field_read_typed(handle)?;
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount.safe_sub(max_locked).unwrap();
            Self::internal_take(delta, api)?;
        }

        // Increase lock count
        locked.amounts.entry(amount).or_default().add_assign(1);

        api.field_write_typed(handle, &locked)?;

        // Issue proof
        Ok(())
    }

    pub fn unlock_amount<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Locked.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedFungibleResource = api.field_read_typed(handle)?;

        let max_locked = locked.amount();
        let cnt = locked
            .amounts
            .remove(&amount)
            .expect("Attempted to unlock an amount that is not locked");
        if cnt > 1 {
            locked.amounts.insert(amount, cnt - 1);
        }

        api.field_write_typed(handle, &locked)?;

        let delta = max_locked.safe_sub(locked.amount()).unwrap();
        Self::internal_put(LiquidFungibleResource::new(delta), api)
    }

    //===================
    // Helper methods
    //===================

    fn liquid_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Liquid.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidFungibleResource = api.field_read_typed(handle)?;
        let amount = substate_ref.amount();
        api.field_close(handle)?;
        Ok(amount)
    }

    fn locked_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Locked.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedFungibleResource = api.field_read_typed(handle)?;
        let amount = substate_ref.amount();
        api.field_close(handle)?;
        Ok(amount)
    }

    fn internal_take<Y>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Liquid.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: LiquidFungibleResource = api.field_read_typed(handle)?;
        let taken = substate.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;
        api.field_write_typed(handle, &substate)?;
        api.field_close(handle)?;
        Ok(taken)
    }

    fn internal_put<Y>(resource: LiquidFungibleResource, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Liquid.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: LiquidFungibleResource = api.field_read_typed(handle)?;
        substate.put(resource);
        api.field_write_typed(handle, &substate)?;
        api.field_close(handle)?;
        Ok(())
    }
}
