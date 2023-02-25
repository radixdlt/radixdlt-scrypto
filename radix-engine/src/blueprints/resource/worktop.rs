use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::resource::{ResourceManager, SysBucket};
use radix_engine_interface::api::substate_api::LockFlags;
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum WorktopError {
    AssertionFailed,
    InsufficientBalance,
}

pub struct WorktopBlueprint;

//==============================================
// Invariant: no empty buckets in the worktop!!!
//==============================================

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

        let resource_address = input.bucket.sys_resource_address(api)?;
        let amount = input.bucket.sys_amount(api)?;

        if amount.is_zero() {
            // TODO: call the burn_empty() method to bypass burn auth
            ResourceManager(resource_address).burn(input.bucket, api)?;
            Ok(IndexedScryptoValue::from_typed(&()))
        } else {
            let worktop_handle = api.sys_lock_substate(
                receiver,
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;
            let worktop: &mut WorktopSubstate = api.kernel_get_substate_ref_mut(worktop_handle)?;
            if let Some(own) = worktop.resources.get(&resource_address).cloned() {
                let existing_bucket = Bucket(own.bucket_id());
                existing_bucket.sys_put(input.bucket, api)?;
            } else {
                worktop
                    .resources
                    .insert(resource_address, Own::Bucket(input.bucket.0));
            }
            api.sys_drop_lock(worktop_handle)?;
            Ok(IndexedScryptoValue::from_typed(&()))
        }
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

        if input.amount.is_zero() {
            let bucket = ResourceManager(input.resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            let worktop_handle = api.sys_lock_substate(
                receiver,
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;
            let worktop: &mut WorktopSubstate = api.kernel_get_substate_ref_mut(worktop_handle)?;
            let bucket = worktop
                .resources
                .get(&input.resource_address)
                .cloned()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ))?;
            let bucket = Bucket(bucket.bucket_id());
            let amount = bucket.sys_amount(api)?;
            let return_bucket = if amount < input.amount {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ));
            } else if amount == input.amount {
                bucket // move
            } else {
                bucket.sys_take(input.amount, api)?
            };
            api.sys_drop_lock(worktop_handle)?;
            Ok(IndexedScryptoValue::from_typed(&return_bucket))
        }
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

        if input.ids.is_empty() {
            let bucket = ResourceManager(input.resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            let worktop_handle = api.sys_lock_substate(
                receiver,
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;
            let worktop: &mut WorktopSubstate = api.kernel_get_substate_ref_mut(worktop_handle)?;
            let bucket = worktop
                .resources
                .get(&input.resource_address)
                .cloned()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ))?;
            let mut bucket = Bucket(bucket.bucket_id());
            let local_ids = bucket.sys_non_fungible_local_ids(api)?;
            let return_bucket = if !local_ids.is_superset(&input.ids) {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ));
            } else if local_ids.len() == input.ids.len() {
                bucket // move
            } else {
                bucket.sys_take_non_fungibles(input.ids, api)?
            };
            api.sys_drop_lock(worktop_handle)?;
            Ok(IndexedScryptoValue::from_typed(&return_bucket))
        }
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

        let worktop_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::MUTABLE,
        )?;
        let worktop: &mut WorktopSubstate = api.kernel_get_substate_ref_mut(worktop_handle)?;
        let return_bucket = if let Some(bucket) = worktop.resources.remove(&input.resource_address)
        {
            Bucket(bucket.bucket_id()) // move
        } else {
            ResourceManager(input.resource_address).new_empty_bucket(api)?
        };
        api.sys_drop_lock(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&return_bucket))
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

        let worktop_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::read_only(),
        )?;
        let worktop: &WorktopSubstate = api.kernel_get_substate_ref(worktop_handle)?;
        let amount = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            Bucket(bucket.bucket_id()).sys_amount(api)?
        } else {
            Decimal::zero()
        };
        if amount.is_zero() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }
        api.sys_drop_lock(worktop_handle)?;
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

        let worktop_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::read_only(),
        )?;
        let worktop: &WorktopSubstate = api.kernel_get_substate_ref(worktop_handle)?;
        let amount = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            Bucket(bucket.bucket_id()).sys_amount(api)?
        } else {
            Decimal::zero()
        };
        if amount < input.amount {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }
        api.sys_drop_lock(worktop_handle)?;
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

        let worktop_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::read_only(),
        )?;
        let worktop: &WorktopSubstate = api.kernel_get_substate_ref(worktop_handle)?;
        let ids = if let Some(bucket) = worktop.resources.get(&input.resource_address) {
            let bucket = Bucket(bucket.bucket_id());
            bucket.sys_non_fungible_local_ids(api)?
        } else {
            BTreeSet::new()
        };
        if !ids.is_superset(&input.ids) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }
        api.sys_drop_lock(worktop_handle)?;
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

        let worktop_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::MUTABLE,
        )?;
        let worktop: &mut WorktopSubstate = api.kernel_get_substate_ref_mut(worktop_handle)?;
        let buckets: Vec<Own> = worktop.resources.values().cloned().collect();
        worktop.resources.clear();
        api.sys_drop_lock(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&buckets))
    }
}
