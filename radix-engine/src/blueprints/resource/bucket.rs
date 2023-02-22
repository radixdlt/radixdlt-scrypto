use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
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
pub struct BucketInfoSubstate {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BucketError {
    InvalidRequestData(DecodeError),

    ResourceError(ResourceError),
    ProofError(ProofError),
    NonFungibleOperationNotSupported,
    MismatchingFungibility,
}

pub struct BucketNode;

impl BucketNode {
    pub fn get_info<Y>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<(ResourceAddress, ResourceType), RuntimeError>
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
        let resource_address = substate_ref.bucket_info().resource_address;
        let resource_type = substate_ref.bucket_info().resource_type;
        api.kernel_drop_lock(handle)?;
        Ok((resource_address, resource_type))
    }

    pub fn liquid_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { .. } => {
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
            ResourceType::NonFungible { .. } => {
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
        }
    }

    pub fn locked_amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { .. } => {
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
            ResourceType::NonFungible { .. } => {
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
        }
    }

    pub fn is_locked<Y>(node_id: RENodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        Ok(!Self::locked_amount(node_id, api)?.is_zero())
    }

    pub fn liquid_non_fungible_ids<Y>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { .. } => Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::NonFungibleOperationNotSupported),
            )),
            ResourceType::NonFungible { .. } => {
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
        }
    }

    pub fn take<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { .. } => {
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
                    .map_err(BucketError::ResourceError)
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::BucketError(e))
                    })?;
                api.kernel_drop_lock(handle)?;
                Ok(LiquidResource::Fungible(taken))
            }
            ResourceType::NonFungible { .. } => {
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
                    .map_err(BucketError::ResourceError)
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::BucketError(e))
                    })?;
                api.kernel_drop_lock(handle)?;
                Ok(LiquidResource::NonFungible(taken))
            }
        }
    }

    pub fn take_non_fungibles<Y>(
        node_id: RENodeId,
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { .. } => Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::NonFungibleOperationNotSupported),
            )),
            ResourceType::NonFungible { .. } => {
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
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::BucketError(e))
                    })?;
                api.kernel_drop_lock(handle)?;
                Ok(taken)
            }
        }
    }

    pub fn put<Y>(
        node_id: RENodeId,
        resource: LiquidResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        if resource.is_empty() {
            return Ok(());
        }

        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { .. } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::LiquidFungible),
                    LockFlags::MUTABLE,
                )?;
                let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
                substate_ref
                    .bucket_liquid_fungible()
                    .put(
                        resource
                            .into_fungible()
                            .ok_or(RuntimeError::ApplicationError(
                                ApplicationError::BucketError(BucketError::MismatchingFungibility),
                            ))?,
                    )
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::BucketError(
                            BucketError::ResourceError(e),
                        ))
                    })?;
                api.kernel_drop_lock(handle)?;
                Ok(())
            }
            ResourceType::NonFungible { .. } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
                    LockFlags::MUTABLE,
                )?;
                let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
                substate_ref
                    .bucket_liquid_non_fungible()
                    .put(
                        resource
                            .into_non_fungibles()
                            .ok_or(RuntimeError::ApplicationError(
                                ApplicationError::BucketError(BucketError::MismatchingFungibility),
                            ))?,
                    )
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::BucketError(
                            BucketError::ResourceError(e),
                        ))
                    })?;
                api.kernel_drop_lock(handle)?;
                Ok(())
            }
        }
    }

    pub fn lock_amount<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<ProofSubstate, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let (resource_address, resource_type) = Self::get_info(node_id, api)?;
        check_amount(amount, resource_type.divisibility()).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;

        match resource_type {
            ResourceType::Fungible { .. } => {
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
                    BucketNode::take(node_id, delta, api)?;
                }

                // Increase lock count
                substate_ref = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
                locked = substate_ref.bucket_locked_fungible();
                locked.amounts.entry(amount).or_default().add_assign(1);

                // Issue proof
                Ok(ProofSubstate::Fungible(
                    FungibleProof::new(
                        resource_address,
                        amount,
                        btreemap!(
                            LocalRef::Bucket(node_id.into()) => amount
                        ),
                    )
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::BucketError(
                            BucketError::ProofError(e),
                        ))
                    })?,
                ))
            }
            ResourceType::NonFungible { .. } => {
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
                    let resource = BucketNode::take(node_id, delta, api)?;
                    let non_fungibles = resource
                        .into_non_fungibles()
                        .expect("Should be non-fungibles");

                    substate_ref = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
                    locked = substate_ref.bucket_locked_non_fungible();
                    for nf in non_fungibles.into_ids() {
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
                Ok(ProofSubstate::NonFungible(
                    NonFungibleProof::new(
                        resource_address,
                        ids_for_proof.clone(),
                        btreemap!(
                            LocalRef::Bucket(node_id.into()) => ids_for_proof
                        ),
                    )
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::BucketError(
                            BucketError::ProofError(e),
                        ))
                    })?,
                ))
            }
        }
    }

    pub fn lock_non_fungibles<Y>(
        node_id: RENodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<ProofSubstate, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let (resource_address, resource_type) = Self::get_info(node_id, api)?;

        match resource_type {
            ResourceType::Fungible { .. } => Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::NonFungibleOperationNotSupported),
            )),
            ResourceType::NonFungible { .. } => {
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
                BucketNode::take_non_fungibles(node_id, &delta, api)?;

                // Increase lock count
                substate_ref = api.kernel_get_substate_ref_mut(handle)?; // grab ref again
                locked = substate_ref.bucket_locked_non_fungible();
                for id in &ids {
                    locked.ids.get_mut(id).unwrap().add_assign(1);
                }

                // Issue proof
                Ok(ProofSubstate::NonFungible(
                    NonFungibleProof::new(
                        resource_address,
                        ids.clone(),
                        btreemap!(
                            LocalRef::Bucket(node_id.into()) => ids
                        ),
                    )
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::BucketError(
                            BucketError::ProofError(e),
                        ))
                    })?,
                ))
            }
        }
    }

    pub fn unlock_amount<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let (resource_address, resource_type) = Self::get_info(node_id, api)?;

        match resource_type {
            ResourceType::Fungible { divisibility } => {
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
                BucketNode::put(
                    node_id,
                    LiquidResource::Fungible(LiquidFungibleResource::new(
                        resource_address,
                        divisibility,
                        delta,
                    )),
                    api,
                )?;

                Ok(())
            }
            ResourceType::NonFungible { .. } => {
                panic!("Attempted to unlock amount on non-fungibles")
            }
        }
    }

    pub fn unlock_non_fungibles<Y>(
        node_id: RENodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let (resource_address, resource_type) = Self::get_info(node_id, api)?;

        match resource_type {
            ResourceType::Fungible { .. } => {
                panic!("Attempted to unlock non-fungibles on fungible")
            }
            ResourceType::NonFungible { id_type } => {
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

                BucketNode::put(
                    node_id,
                    LiquidResource::NonFungible(LiquidNonFungibleResource::new(
                        resource_address,
                        id_type,
                        liquid_non_fungibles,
                    )),
                    api,
                )?;

                Ok(())
            }
        }
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

        // Take
        let taken = BucketNode::take(receiver, input.amount, api)?;

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            match taken {
                LiquidResource::Fungible(f) => RENodeInit::FungibleBucket(f),
                LiquidResource::NonFungible(nf) => RENodeInit::NonFungibleBucket(nf),
            },
            BTreeMap::new(),
        )?;
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

        // Take
        let taken = BucketNode::take_non_fungibles(receiver, &input.ids, api)?;

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::NonFungibleBucket(taken),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
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
        let other_bucket: LiquidResource = api
            .kernel_drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();

        // Put
        BucketNode::put(receiver, other_bucket, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
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

        let amount =
            BucketNode::locked_amount(receiver, api)? + BucketNode::liquid_amount(receiver, api)?;
        let proof = BucketNode::lock_amount(receiver, amount, api)?;

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
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

        let ids: BTreeSet<NonFungibleLocalId> = BucketNode::liquid_non_fungible_ids(receiver, api)?;

        Ok(IndexedScryptoValue::from_typed(&ids))
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

        let amount: Decimal = BucketNode::liquid_amount(receiver, api)?;

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

        let resource_address: ResourceAddress = BucketNode::get_info(receiver, api)?.0;

        Ok(IndexedScryptoValue::from_typed(&resource_address))
    }
}
