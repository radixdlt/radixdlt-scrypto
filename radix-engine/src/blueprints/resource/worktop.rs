use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::resource::{ResourceManager, SysBucket};
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, ScryptoSbor)]
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
    pub(crate) fn drop<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopDropInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let mut node = api.kernel_drop_node(&RENodeId::Object(input.worktop.id()))?;
        let mut worktop_substates = node.substates.remove(&NodeModuleId::SELF).unwrap();
        let substate = worktop_substates
            .remove(&SubstateOffset::Worktop(WorktopOffset::Worktop))
            .unwrap();
        let worktop: WorktopSubstate = substate.into();
        for (_, bucket) in worktop.resources {
            let bucket = Bucket(bucket.bucket_id());
            bucket.sys_drop_empty(api)?;
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn put<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopPutInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let resource_address = input.bucket.sys_resource_address(api)?;
        let amount = input.bucket.sys_amount(api)?;

        if amount.is_zero() {
            input.bucket.sys_burn(api)?;
            Ok(IndexedScryptoValue::from_typed(&()))
        } else {
            let worktop_handle = api.sys_lock_substate(
                receiver.clone(),
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;
            let worktop: &mut WorktopSubstate = api.kernel_get_substate_ref_mut(worktop_handle)?;
            if let Some(own) = worktop.resources.get(&resource_address).cloned() {
                Bucket(own.bucket_id()).sys_put(input.bucket, api)?;
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
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopTakeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let resource_address = input.resource_address;
        let amount = input.amount;

        if amount.is_zero() {
            let bucket = ResourceManager(resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            let worktop_handle = api.sys_lock_substate(
                receiver.clone(),
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;
            let mut worktop: &mut WorktopSubstate =
                api.kernel_get_substate_ref_mut(worktop_handle)?;
            let existing_bucket = Bucket(
                worktop
                    .resources
                    .get(&resource_address)
                    .cloned()
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                    ))?
                    .bucket_id(),
            );
            let existing_amount = existing_bucket.sys_amount(api)?;

            if existing_amount < amount {
                Err(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ))
            } else if existing_amount == amount {
                // Move
                worktop = api.kernel_get_substate_ref_mut(worktop_handle)?;
                worktop.resources.remove(&resource_address);
                api.sys_drop_lock(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&existing_bucket))
            } else {
                let bucket = existing_bucket.sys_take(amount, api)?;
                api.sys_drop_lock(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&bucket))
            }
        }
    }

    pub(crate) fn take_non_fungibles<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopTakeNonFungiblesInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let resource_address = input.resource_address;
        let ids = input.ids;

        if ids.is_empty() {
            let bucket = ResourceManager(resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            let worktop_handle = api.sys_lock_substate(
                receiver.clone(),
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;
            let mut worktop: &mut WorktopSubstate =
                api.kernel_get_substate_ref_mut(worktop_handle)?;
            let existing_bucket = Bucket(
                worktop
                    .resources
                    .get(&resource_address)
                    .cloned()
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                    ))?
                    .bucket_id(),
            );
            let existing_non_fungibles = existing_bucket.sys_non_fungible_local_ids(api)?;

            if !existing_non_fungibles.is_superset(&ids) {
                Err(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ))
            } else if existing_non_fungibles.len() == ids.len() {
                // Move
                worktop = api.kernel_get_substate_ref_mut(worktop_handle)?;
                worktop.resources.remove(&resource_address);
                api.sys_drop_lock(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&existing_bucket))
            } else {
                let bucket = existing_bucket.sys_take_non_fungibles(ids, api)?;
                api.sys_drop_lock(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&bucket))
            }
        }
    }

    pub(crate) fn take_all<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopTakeAllInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let worktop_handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::Worktop(WorktopOffset::Worktop),
            LockFlags::MUTABLE,
        )?;
        let worktop: &mut WorktopSubstate = api.kernel_get_substate_ref_mut(worktop_handle)?;
        if let Some(bucket) = worktop.resources.remove(&input.resource_address) {
            // Move
            api.sys_drop_lock(worktop_handle)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            api.sys_drop_lock(worktop_handle)?;
            let bucket = ResourceManager(input.resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        }
    }

    pub(crate) fn assert_contains<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopAssertContainsInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let worktop_handle = api.sys_lock_substate(
            receiver.clone(),
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
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopAssertContainsAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let worktop_handle = api.sys_lock_substate(
            receiver.clone(),
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
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: WorktopAssertContainsNonFungiblesInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let worktop_handle = api.sys_lock_substate(
            receiver.clone(),
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
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: WorktopDrainInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let worktop_handle = api.sys_lock_substate(
            receiver.clone(),
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
