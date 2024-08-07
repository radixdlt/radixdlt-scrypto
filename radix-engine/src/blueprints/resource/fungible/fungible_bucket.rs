use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use radix_engine_interface::api::{
    FieldValue, LockFlags, SystemApi, ACTOR_REF_OUTER, ACTOR_STATE_OUTER_OBJECT, ACTOR_STATE_SELF,
};
use radix_engine_interface::blueprints::resource::*;
use radix_native_sdk::runtime::Runtime;

pub struct FungibleBucketBlueprint;

impl FungibleBucketBlueprint {
    fn get_divisibility<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<u8, RuntimeError> {
        let divisibility_handle = api.actor_open_field(
            ACTOR_STATE_OUTER_OBJECT,
            FungibleResourceManagerField::Divisibility.into(),
            LockFlags::read_only(),
        )?;
        let divisibility: u8 = api
            .field_read_typed::<FungibleResourceManagerDivisibilityFieldPayload>(
                divisibility_handle,
            )?
            .fully_update_and_into_latest_version();
        api.field_close(divisibility_handle)?;
        Ok(divisibility)
    }

    pub fn take<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::take_advanced(amount, WithdrawStrategy::Exact, api)
    }

    pub fn take_advanced<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let taken = {
            let divisibility = Self::get_divisibility(api)?;
            // Apply withdraw strategy
            let amount = amount
                .for_withdrawal(divisibility, withdraw_strategy)
                .ok_or(BucketError::DecimalOverflow)?;

            // Check amount
            if !(check_fungible_amount(&amount, divisibility)) {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::BucketError(BucketError::InvalidAmount(amount)),
                ));
            }

            Self::internal_take(amount, api)?
        };

        // Create node
        let bucket = FungibleResourceManagerBlueprint::create_bucket(taken.amount(), api)?;

        Ok(bucket)
    }

    pub fn put<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // This will fail if bucket is not an inner object of the current fungible resource
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

    pub fn get_amount<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Decimal, RuntimeError> {
        Self::liquid_amount(api)?
            .checked_add(Self::locked_amount(api)?)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::DecimalOverflow),
            ))
    }

    pub fn get_resource_address<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError> {
        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_OUTER)?.into());

        Ok(resource_address)
    }

    pub fn create_proof_of_amount<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError> {
        let divisibility = Self::get_divisibility(api)?;
        if !check_fungible_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::InvalidAmount(amount)),
            ));
        }

        Self::lock_amount(amount, api)?;

        let receiver = Runtime::get_node_id(api)?;
        let proof_info = ProofMoveableSubstate { restricted: false };
        let proof_evidence = FungibleProofSubstate::new(
            amount,
            indexmap!(
                LocalRef::Bucket(Reference(receiver)) => amount
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?;
        let proof_id = api.new_simple_object(
            FUNGIBLE_PROOF_BLUEPRINT,
            indexmap! {
                FungibleProofField::Moveable.field_index() => FieldValue::new(&proof_info),
                FungibleProofField::ProofRefs.field_index() => FieldValue::new(&proof_evidence),
            },
        )?;

        Ok(Proof(Own(proof_id)))
    }

    pub fn create_proof_of_all<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Proof, RuntimeError> {
        Self::create_proof_of_amount(Self::get_amount(api)?, api)
    }

    //===================
    // Protected method
    //===================

    pub fn lock_amount<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Locked.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedFungibleResource = api.field_read_typed(handle)?;
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount
                .checked_sub(max_locked)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::BucketError(BucketError::DecimalOverflow),
                ))?;
            Self::internal_take(delta, api)?;
        }

        // Increase lock count
        locked.amounts.entry(amount).or_default().add_assign(1);

        api.field_write_typed(handle, &locked)?;

        // Issue proof
        Ok(())
    }

    pub fn unlock_amount<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleBucketField::Locked.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedFungibleResource = api.field_read_typed(handle)?;

        let max_locked = locked.amount();
        let cnt = locked
            .amounts
            .swap_remove(&amount)
            .expect("Attempted to unlock an amount that is not locked");
        if cnt > 1 {
            locked.amounts.insert(amount, cnt - 1);
        }

        api.field_write_typed(handle, &locked)?;

        let delta =
            max_locked
                .checked_sub(locked.amount())
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::BucketError(BucketError::DecimalOverflow),
                ))?;
        Self::internal_put(LiquidFungibleResource::new(delta), api)
    }

    //===================
    // Helper methods
    //===================

    fn liquid_amount<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Decimal, RuntimeError> {
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

    fn locked_amount<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Decimal, RuntimeError> {
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

    fn internal_take<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidFungibleResource, RuntimeError> {
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

    fn internal_put<Y: SystemApi<RuntimeError>>(
        resource: LiquidFungibleResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
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
