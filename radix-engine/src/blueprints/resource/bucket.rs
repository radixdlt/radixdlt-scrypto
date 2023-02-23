use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::heap::{DroppedBucket, DroppedBucketResource};
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;
use sbor::rust::ops::SubAssign;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BucketError {
    InvalidRequestData(DecodeError),

    ResourceError(ResourceError),
    ProofError(ProofError),
    NonFungibleOperationNotSupported,
    MismatchingResource,
    InvalidAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct BucketInfoSubstate {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
}

impl BucketInfoSubstate {
    pub fn of<Y>(node_id: RENodeId, api: &mut Y) -> Result<Self, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Info),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let info = substate_ref.bucket_info().clone();
        api.kernel_drop_lock(handle)?;
        Ok(info)
    }
}

pub struct FungibleBucket;

impl FungibleBucket {
    pub fn liquid_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::LiquidFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.bucket_liquid_fungible().amount();
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
            SubstateOffset::Bucket(BucketOffset::LockedFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.bucket_locked_fungible().amount();
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
            SubstateOffset::Bucket(BucketOffset::LiquidFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let taken = substate_ref
            .bucket_liquid_fungible()
            .take_by_amount(amount)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::BucketError(
                    BucketError::ResourceError(e),
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
            SubstateOffset::Bucket(BucketOffset::LiquidFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        substate_ref
            .bucket_liquid_fungible()
            .put(resource)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::BucketError(
                    BucketError::ResourceError(e),
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
            SubstateOffset::Bucket(BucketOffset::LockedFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let mut locked = substate_ref.bucket_locked_fungible();
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            FungibleBucket::take(node_id, delta, api)?;
        }

        // Increase lock count
        substate_ref = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
        locked = substate_ref.bucket_locked_fungible();
        locked.amounts.entry(amount).or_default().add_assign(1);

        // Issue proof
        Ok(FungibleProof::new(
            amount,
            btreemap!(
                LocalRef::Bucket(node_id.into()) => amount
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
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
            SubstateOffset::Bucket(BucketOffset::LockedFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let locked = substate_ref.bucket_locked_fungible();

        let max_locked = locked.amount();
        locked
            .amounts
            .get_mut(&amount)
            .expect("Attempted to unlock an amount that is not locked in container")
            .sub_assign(1);

        let delta = max_locked - locked.amount();
        FungibleBucket::put(node_id, LiquidFungibleResource::new(delta), api)
    }
}

pub struct NonFungibleBucket;

impl NonFungibleBucket {
    pub fn liquid_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.bucket_liquid_non_fungible().amount();
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
            SubstateOffset::Bucket(BucketOffset::LockedNonFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let amount = substate_ref.bucket_locked_non_fungible().amount();
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
            SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let ids = substate_ref.bucket_liquid_non_fungible().ids().clone();
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
            SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let taken = substate_ref
            .bucket_liquid_non_fungible()
            .take_by_amount(amount)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::BucketError(
                    BucketError::ResourceError(e),
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
            SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let taken = substate_ref
            .bucket_liquid_non_fungible()
            .take_by_ids(ids)
            .map_err(BucketError::ResourceError)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::BucketError(e)))?;
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
            SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        substate_ref
            .bucket_liquid_non_fungible()
            .put(resource)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::BucketError(
                    BucketError::ResourceError(e),
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
            SubstateOffset::Bucket(BucketOffset::LockedNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let mut locked = substate_ref.bucket_locked_non_fungible();
        let max_locked: Decimal = locked.ids.len().into();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            let resource = NonFungibleBucket::take(node_id, delta, api)?;

            substate_ref = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
            locked = substate_ref.bucket_locked_non_fungible();
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
                LocalRef::Bucket(node_id.into()) => ids_for_proof
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
            SubstateOffset::Bucket(BucketOffset::LockedNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let mut locked = substate_ref.bucket_locked_non_fungible();

        // Take from liquid if needed
        let delta: BTreeSet<NonFungibleLocalId> = ids
            .iter()
            .cloned()
            .filter(|id| !locked.ids.contains_key(id))
            .collect();
        NonFungibleBucket::take_non_fungibles(node_id, &delta, api)?;

        // Increase lock count
        substate_ref = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
        locked = substate_ref.bucket_locked_non_fungible();
        for id in &ids {
            locked.ids.get_mut(id).unwrap().add_assign(1);
        }

        // Issue proof
        Ok(NonFungibleProof::new(
            ids.clone(),
            btreemap!(
                LocalRef::Bucket(node_id.into()) => ids
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
            SubstateOffset::Bucket(BucketOffset::LockedNonFungible),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        let locked = substate_ref.bucket_locked_non_fungible();

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

        NonFungibleBucket::put(
            node_id,
            LiquidNonFungibleResource::new(liquid_non_fungibles),
            api,
        )
    }
}

pub struct BucketBlueprint;

impl BucketBlueprint {
    pub fn take<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: BucketTakeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        // Check amount
        let info = BucketInfoSubstate::of(receiver, api)?;
        if !info.resource_type.check_amount(input.amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::InvalidAmount),
            ));
        }

        let node_id = if info.resource_type.is_fungible() {
            // Take
            let taken = FungibleBucket::take(receiver, input.amount, api)?;

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
            let taken = NonFungibleBucket::take(receiver, input.amount, api)?;

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
        // TODO: Remove decode/encode mess
        let input: BucketTakeNonFungiblesInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = BucketInfoSubstate::of(receiver, api)?;

        if info.resource_type.is_fungible() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::NonFungibleOperationNotSupported),
            ));
        } else {
            // Take
            let taken = NonFungibleBucket::take_non_fungibles(receiver, &input.ids, api)?;

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
        // TODO: Remove decode/encode mess
        let input: BucketPutInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        // Drop other bucket
        let other_bucket: DroppedBucket = api
            .kernel_drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();

        // Check resource address
        let info = BucketInfoSubstate::of(receiver, api)?;
        if info.resource_address != other_bucket.info.resource_address {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::MismatchingResource),
            ));
        }

        // Put
        match other_bucket.resource {
            DroppedBucketResource::Fungible(r) => {
                FungibleBucket::put(receiver, r, api)?;
            }
            DroppedBucketResource::NonFungible(r) => {
                NonFungibleBucket::put(receiver, r, api)?;
            }
        }
        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn get_non_fungible_local_ids<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: BucketGetNonFungibleLocalIdsInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = BucketInfoSubstate::of(receiver, api)?;
        if info.resource_type.is_fungible() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::NonFungibleOperationNotSupported),
            ));
        } else {
            let ids: BTreeSet<NonFungibleLocalId> =
                NonFungibleBucket::liquid_non_fungible_local_ids(receiver, api)?;
            Ok(IndexedScryptoValue::from_typed(&ids))
        }
    }

    pub fn get_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: BucketGetAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = BucketInfoSubstate::of(receiver, api)?;
        let amount = if info.resource_type.is_fungible() {
            FungibleBucket::liquid_amount(receiver, api)?
        } else {
            NonFungibleBucket::liquid_amount(receiver, api)?
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
        // TODO: Remove decode/encode mess
        let _input: BucketGetResourceAddressInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = BucketInfoSubstate::of(receiver, api)?;

        Ok(IndexedScryptoValue::from_typed(&info.resource_address))
    }

    pub fn create_proof<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: BucketCreateProofInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let info = BucketInfoSubstate::of(receiver, api)?;
        let node_id = if info.resource_type.is_fungible() {
            let amount = FungibleBucket::locked_amount(receiver, api)?
                + FungibleBucket::liquid_amount(receiver, api)?;
            let proof = FungibleBucket::lock_amount(receiver, amount, api)?;

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
            let amount = NonFungibleBucket::locked_amount(receiver, api)?
                + NonFungibleBucket::liquid_amount(receiver, api)?;
            let proof = NonFungibleBucket::lock_amount(receiver, amount, api)?;

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
        let input: BucketLockAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        FungibleBucket::lock_amount(receiver, input.amount, api)?;

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
        let input: BucketLockNonFungiblesInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        NonFungibleBucket::lock_non_fungibles(receiver, input.local_ids, api)?;

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
        let input: BucketUnlockAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        FungibleBucket::unlock_amount(receiver, input.amount, api)?;

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
        let input: BucketUnlockNonFungiblesInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        NonFungibleBucket::unlock_non_fungibles(receiver, input.local_ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
