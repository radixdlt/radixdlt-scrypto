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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct BucketInfoSubstate {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BucketError {
    InvalidDivisibility,
    InvalidRequestData(DecodeError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ResourceError(ResourceError),
    ProofError(ProofError),
    CouldNotCreateProof,

    NonFungibleOperationOnFungible,
    MismatchingFungibility,
}

pub struct BucketNode;

impl BucketNode {
    pub(crate) fn get_info<Y>(
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

    pub(crate) fn is_locked<Y>(node_id: RENodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::LockedFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let locked = substate_ref.bucket_locked_fungible().is_locked();
                api.kernel_drop_lock(handle)?;
                Ok(locked)
            }
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::LockedNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let locked = substate_ref.bucket_locked_non_fungible().is_locked();
                api.kernel_drop_lock(handle)?;
                Ok(locked)
            }
        }
    }

    pub(crate) fn amount<Y>(node_id: RENodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => {
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
            ResourceType::NonFungible { id_type } => {
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

    pub(crate) fn non_fungible_ids<Y>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::NonFungibleOperationOnFungible),
            )),
            ResourceType::NonFungible { id_type } => {
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

    pub(crate) fn take<Y>(
        node_id: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::LiquidFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
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
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
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

    pub(crate) fn take_non_fungibles<Y>(
        node_id: RENodeId,
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => Err(RuntimeError::ApplicationError(
                ApplicationError::BucketError(BucketError::NonFungibleOperationOnFungible),
            )),
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
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

    pub(crate) fn put<Y>(
        node_id: RENodeId,
        resource: LiquidResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        match Self::get_info(node_id, api)?.1 {
            ResourceType::Fungible { divisibility } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::LiquidFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let taken = substate_ref
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
            ResourceType::NonFungible { id_type } => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let taken =
                    substate_ref
                        .bucket_liquid_non_fungible()
                        .put(resource.into_non_fungibles().ok_or(
                            RuntimeError::ApplicationError(ApplicationError::BucketError(
                                BucketError::MismatchingFungibility,
                            )),
                        )?)
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
}

pub struct BucketBlueprint;

impl BucketBlueprint {
    pub(crate) fn take<Y>(
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
        let info = BucketInfoSubstate {
            resource_address: taken.resource_address(),
            resource_type: taken.resource_type(),
        };

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            match taken {
                LiquidResource::Fungible(f) => RENodeInit::FungibleBucket(info, f),
                LiquidResource::NonFungible(nf) => RENodeInit::NonFungibleBucket(info, nf),
            },
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
    }

    pub(crate) fn take_non_fungibles<Y>(
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
        let info = BucketInfoSubstate {
            resource_address: taken.resource_address(),
            resource_type: taken.resource_type(),
        };

        // Create node
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::NonFungibleBucket(info, taken),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
    }

    pub(crate) fn put<Y>(
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

    pub(crate) fn create_proof<Y>(
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

        let bucket_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::MUTABLE,
        )?;

        let mut substate_mut = api.kernel_get_substate_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket_info();
        let proof = bucket.create_proof(receiver.into()).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?;

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn get_non_fungible_local_ids<Y>(
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

        let ids: BTreeSet<NonFungibleLocalId> = BucketNode::non_fungible_ids(receiver, api)?;

        Ok(IndexedScryptoValue::from_typed(&ids))
    }

    pub(crate) fn get_amount<Y>(
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

        let amount: Decimal = BucketNode::amount(receiver, api)?;

        Ok(IndexedScryptoValue::from_typed(&amount))
    }

    pub(crate) fn get_resource_address<Y>(
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
