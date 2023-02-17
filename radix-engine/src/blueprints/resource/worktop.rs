use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::resource::{ResourceManager, SysBucket};
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{RENodeId, SubstateOffset, WorktopOffset};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug)]
pub struct WorktopSubstate {
    pub resources: BTreeMap<ResourceAddress, Own>,
}

impl WorktopSubstate {
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum WorktopError {
    AssertionFailed,
}

pub struct WorktopBlueprint;

impl WorktopBlueprint {
    pub(crate) fn put<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopPutInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::MUTABLE,
        )?;

        let resource_address = input.bucket.sys_resource_address(api)?;

        let mut substate_mut = api.kernel_get_substate_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();

        if let Some(own) = worktop.resources.get(&resource_address).cloned() {
            let existing_bucket = Bucket(own.bucket_id());
            existing_bucket.sys_put(input.bucket, api)?;
        } else {
            worktop
                .resources
                .insert(resource_address, Own::Bucket(input.bucket.0));
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn take<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopTakeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::MUTABLE,
        )?;

        let mut substate_mut = api.kernel_get_substate_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();
        let bucket = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            bucket
        } else {
            let resman = ResourceManager(input.resource_address);
            let bucket = Own::Bucket(resman.new_empty_bucket(api)?.0);

            let mut substate_mut = api.kernel_get_substate_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            worktop.resources.insert(input.resource_address, bucket);
            worktop
                .resources
                .get(&input.resource_address)
                .cloned()
                .unwrap()
        };

        let rtn_bucket = Bucket(bucket.bucket_id()).sys_take(input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&rtn_bucket))
    }

    pub(crate) fn take_all<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopTakeAllInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::MUTABLE,
        )?;
        let mut substate_mut = api.kernel_get_substate_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();

        let rtn_bucket = if let Some(bucket) = worktop.resources.get(&input.resource_address) {
            let bucket = Bucket(bucket.bucket_id());
            let amount = bucket.sys_amount(api)?;
            let rtn_bucket = bucket.sys_take(amount, api)?;
            rtn_bucket
        } else {
            let resman = ResourceManager(input.resource_address);
            resman.new_empty_bucket(api)?
        };

        Ok(IndexedScryptoValue::from_typed(&rtn_bucket))
    }

    pub(crate) fn take_non_fungibles<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopTakeNonFungiblesInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::MUTABLE,
        )?;
        let mut substate_mut = api.kernel_get_substate_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();

        let bucket = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            bucket
        } else {
            let resman = ResourceManager(input.resource_address);
            let bucket = Own::Bucket(resman.new_empty_bucket(api)?.0);

            let mut substate_mut = api.kernel_get_substate_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            worktop.resources.insert(input.resource_address, bucket);
            worktop
                .resources
                .get(&input.resource_address)
                .cloned()
                .unwrap()
        };

        let mut bucket = Bucket(bucket.bucket_id());
        let rtn_bucket = bucket.sys_take_non_fungibles(input.ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&rtn_bucket))
    }

    pub(crate) fn assert_contains<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopAssertContainsInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::read_only(),
        )?;

        let substate_ref = api.kernel_get_substate_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();

        let total_amount =
            if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
                Bucket(bucket.bucket_id()).sys_amount(api)?
            } else {
                Decimal::zero()
            };
        if total_amount.is_zero() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn assert_contains_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopAssertContainsAmountInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::read_only(),
        )?;

        let substate_ref = api.kernel_get_substate_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();
        let total_amount =
            if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
                Bucket(bucket.bucket_id()).sys_amount(api)?
            } else {
                Decimal::zero()
            };
        if total_amount < input.amount {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn assert_contains_non_fungibles<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopAssertContainsNonFungiblesInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::read_only(),
        )?;

        let substate_ref = api.kernel_get_substate_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();

        let ids = if let Some(bucket) = worktop.resources.get(&input.resource_address) {
            let bucket = Bucket(bucket.bucket_id());
            bucket.sys_total_ids(api)?
        } else {
            BTreeSet::new()
        };
        if !ids.is_superset(&input.ids) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn drain<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: WorktopDrainInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::MUTABLE,
        )?;
        let mut buckets = Vec::new();
        let mut substate_mut = api.kernel_get_substate_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();
        let bucket_ids: Vec<BucketId> = worktop
            .resources
            .iter()
            .map(|(_, own)| own.bucket_id())
            .collect();
        for bucket_id in bucket_ids {
            let bucket = Bucket(bucket_id);
            let amount = bucket.sys_amount(api)?;
            let bucket = bucket.sys_take(amount, api)?;
            buckets.push(bucket);
        }

        Ok(IndexedScryptoValue::from_typed(&buckets))
    }
}
