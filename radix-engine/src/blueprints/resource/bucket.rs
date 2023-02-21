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

        let bucket_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::MUTABLE,
        )?;

        let mut substate_mut = api.kernel_get_substate_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket_info();
        let container = bucket.take(input.amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;

        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(container)),
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

        let bucket_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::MUTABLE,
        )?;

        let mut substate_mut = api.kernel_get_substate_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket_info();
        let container = bucket.take_non_fungibles(&input.ids).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;

        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(container)),
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

        let bucket_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::MUTABLE,
        )?;

        let other_bucket = api
            .kernel_drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();
        let mut substate_mut = api.kernel_get_substate_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket_info();
        bucket.put(other_bucket).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;

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

        let bucket_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(bucket_handle)?;
        let bucket = substate_ref.bucket_info();
        let ids = bucket.total_ids().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceError(e),
            ))
        })?;

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

        let bucket_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::read_only(),
        )?;

        let substate = api.kernel_get_substate_ref(bucket_handle)?;
        let bucket = substate.bucket_info();
        Ok(IndexedScryptoValue::from_typed(&bucket.total_amount()))
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

        let bucket_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::read_only(),
        )?;

        let substate = api.kernel_get_substate_ref(bucket_handle)?;
        let bucket = substate.bucket_info();

        Ok(IndexedScryptoValue::from_typed(&bucket.resource_address()))
    }
}
