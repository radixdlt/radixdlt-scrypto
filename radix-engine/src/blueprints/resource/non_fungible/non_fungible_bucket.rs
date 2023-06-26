use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::resource::*;

pub struct NonFungibleBucket;

impl NonFungibleBucket {
    pub fn liquid_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleBucketField::Liquid.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidNonFungibleResource = api.field_lock_read_typed(handle)?;
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
            NonFungibleBucketField::Locked.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedNonFungibleResource = api.field_lock_read_typed(handle)?;
        let amount = substate_ref.amount();
        api.field_lock_release(handle)?;
        Ok(amount)
    }

    pub fn liquid_non_fungible_local_ids<Y>(
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleBucketField::Liquid.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidNonFungibleResource = api.field_lock_read_typed(handle)?;
        let ids = substate_ref.ids().clone();
        api.field_lock_release(handle)?;
        Ok(ids)
    }

    pub fn locked_non_fungible_local_ids<Y>(
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleBucketField::Locked.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedNonFungibleResource = api.field_lock_read_typed(handle)?;
        let ids = substate_ref.ids();
        api.field_lock_release(handle)?;
        Ok(ids)
    }

    pub fn take<Y>(amount: Decimal, api: &mut Y) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleBucketField::Liquid.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: LiquidNonFungibleResource = api.field_lock_read_typed(handle)?;
        let taken = substate.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;
        api.field_lock_write_typed(handle, &substate)?;
        api.field_lock_release(handle)?;
        Ok(taken)
    }

    pub fn take_non_fungibles<Y>(
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleBucketField::Liquid.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: LiquidNonFungibleResource = api.field_lock_read_typed(handle)?;
        let taken = substate
            .take_by_ids(ids)
            .map_err(BucketError::ResourceError)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::BucketError(e)))?;
        api.field_lock_write_typed(handle, &substate)?;
        api.field_lock_release(handle)?;
        Ok(taken)
    }

    pub fn put<Y>(resource: LiquidNonFungibleResource, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleBucketField::Liquid.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: LiquidNonFungibleResource = api.field_lock_read_typed(handle)?;
        substate.put(resource).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;
        api.field_lock_write_typed(handle, &substate)?;
        api.field_lock_release(handle)?;
        Ok(())
    }

    // protected method
    pub fn lock_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<NonFungibleProofSubstate, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleBucketField::Locked.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.field_lock_read_typed(handle)?;
        let max_locked: Decimal = locked.ids.len().into();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            let resource = NonFungibleBucket::take(delta, api)?;

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

        api.field_lock_write_typed(handle, &locked)?;

        // Issue proof
        Ok(NonFungibleProofSubstate::new(
            ids_for_proof.clone(),
            btreemap!(
                LocalRef::Bucket(Reference(receiver.clone())) => ids_for_proof
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?)
    }

    // protected method
    pub fn lock_non_fungibles<Y>(
        receiver: &NodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<NonFungibleProofSubstate, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleBucketField::Locked.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.field_lock_read_typed(handle)?;

        // Take from liquid if needed
        let delta: BTreeSet<NonFungibleLocalId> = ids
            .iter()
            .cloned()
            .filter(|id| !locked.ids.contains_key(id))
            .collect();
        NonFungibleBucket::take_non_fungibles(&delta, api)?;

        // Increase lock count
        for id in &ids {
            locked.ids.entry(id.clone()).or_default().add_assign(1);
        }

        api.field_lock_write_typed(handle, &locked)?;

        // Issue proof
        Ok(NonFungibleProofSubstate::new(
            ids.clone(),
            btreemap!(
                LocalRef::Bucket(Reference(receiver.clone())) => ids
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?)
    }

    // protected method
    pub fn unlock_non_fungibles<Y>(
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleBucketField::Locked.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.field_lock_read_typed(handle)?;

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

        api.field_lock_write_typed(handle, &locked)?;

        NonFungibleBucket::put(LiquidNonFungibleResource::new(liquid_non_fungibles), api)
    }
}

pub struct NonFungibleBucketBlueprint;

impl NonFungibleBucketBlueprint {
    pub fn take<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: BucketTakeInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        // Check amount
        if !check_non_fungible_amount(&input.amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::InvalidAmount),
            ));
        }

        // Take
        let taken = NonFungibleBucket::take(input.amount, api)?;

        // Create node
        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    pub fn take_non_fungibles<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: BucketTakeNonFungiblesInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        // Take
        let taken = NonFungibleBucket::take_non_fungibles(&input.ids, api)?;

        // Create node
        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    pub fn put<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: BucketPutInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        // Drop other bucket
        let other_bucket = drop_non_fungible_bucket(input.bucket.0.as_node_id(), api)?;

        // Put
        NonFungibleBucket::put(other_bucket.liquid, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn get_non_fungible_local_ids<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: BucketGetNonFungibleLocalIdsInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let mut ids = NonFungibleBucket::liquid_non_fungible_local_ids(api)?;
        ids.extend(NonFungibleBucket::locked_non_fungible_local_ids(api)?);
        Ok(IndexedScryptoValue::from_typed(&ids))
    }

    pub fn get_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let amount =
            NonFungibleBucket::liquid_amount(api)? + NonFungibleBucket::locked_amount(api)?;

        Ok(amount)
    }

    pub fn get_resource_address<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: BucketGetResourceAddressInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_info()?.get_outer_object().into());

        Ok(IndexedScryptoValue::from_typed(&resource_address))
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
        if !check_non_fungible_amount(&amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::InvalidAmount),
            ));
        }

        let proof_info = ProofMoveableSubstate { restricted: false };
        let proof = NonFungibleBucket::lock_amount(receiver, amount, api)?;
        let proof_id = api.new_simple_object(
            NON_FUNGIBLE_PROOF_BLUEPRINT,
            vec![
                scrypto_encode(&proof_info).unwrap(),
                scrypto_encode(&proof).unwrap(),
            ],
        )?;

        Ok(Proof(Own(proof_id)))
    }

    pub fn create_proof_of_non_fungibles<Y>(
        receiver: &NodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let proof_info = ProofMoveableSubstate { restricted: false };
        let proof = NonFungibleBucket::lock_non_fungibles(receiver, ids, api)?;
        let proof_id = api.new_simple_object(
            NON_FUNGIBLE_PROOF_BLUEPRINT,
            vec![
                scrypto_encode(&proof_info).unwrap(),
                scrypto_encode(&proof).unwrap(),
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

    pub fn lock_non_fungibles<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let input: NonFungibleBucketLockNonFungiblesInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        NonFungibleBucket::lock_non_fungibles(receiver, input.local_ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn unlock_non_fungibles<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let input: NonFungibleBucketUnlockNonFungiblesInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        NonFungibleBucket::unlock_non_fungibles(input.local_ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
