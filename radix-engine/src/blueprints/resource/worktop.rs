use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::system_callback::SystemLockData;
use crate::system::system_substates::FieldSubstate;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::{SystemApi, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::resource::*;
use radix_native_sdk::resource::{NativeBucket, NativeNonFungibleBucket, ResourceManager};

#[derive(Debug, ScryptoSbor)]
pub struct WorktopSubstate {
    pub resources: IndexMap<ResourceAddress, Own>,
}

impl WorktopSubstate {
    pub fn new() -> Self {
        Self {
            resources: index_map_new(),
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
// Invariant: no empty buckets in the worktop!
//==============================================

impl WorktopBlueprint {
    pub(crate) fn drop<Y: SystemApi<RuntimeError> + KernelSubstateApi<SystemLockData>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        // TODO: add `drop` callback for drop atomicity, which will remove the necessity of kernel api.

        let input: WorktopDropInput = input
            .into_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        // Detach buckets from worktop
        let handle = api.kernel_open_substate(
            input.worktop.0.as_node_id(),
            MAIN_BASE_PARTITION,
            &WorktopField::Worktop.into(),
            LockFlags::MUTABLE,
            SystemLockData::Default,
        )?;
        let mut worktop = api
            .kernel_read_substate(handle)?
            .into_typed::<FieldSubstate<WorktopSubstate>>()
            .unwrap()
            .into_payload();
        let resources = core::mem::replace(&mut worktop.resources, index_map_new());
        api.kernel_write_substate(
            handle,
            IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(worktop)),
        )?;
        api.kernel_close_substate(handle)?;

        // Recursively drop buckets
        for (_, bucket) in resources {
            let bucket = Bucket(bucket);
            bucket.drop_empty(api)?;
        }

        // Destroy self
        api.drop_object(input.worktop.0.as_node_id())?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn put<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        let input: WorktopPutInput = input
            .into_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let resource_address = input.bucket.resource_address(api)?;
        let amount = input.bucket.amount(api)?;

        if amount.is_zero() {
            input.bucket.drop_empty(api)?;
            Ok(IndexedScryptoValue::from_typed(&()))
        } else {
            let worktop_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                WorktopField::Worktop.into(),
                LockFlags::MUTABLE,
            )?;
            let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
            if let Some(own) = worktop.resources.get(&resource_address).cloned() {
                Bucket(own).put(input.bucket, api)?;
            } else {
                worktop.resources.insert(resource_address, input.bucket.0);
                api.field_write_typed(worktop_handle, &worktop)?;
            }
            api.field_close(worktop_handle)?;
            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }

    pub(crate) fn take<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        let input: WorktopTakeInput = input
            .into_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let resource_address = input.resource_address;
        let amount = input.amount;

        if amount.is_zero() {
            let bucket = ResourceManager(resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            let worktop_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                WorktopField::Worktop.into(),
                LockFlags::MUTABLE,
            )?;
            let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
            let existing_bucket = Bucket(worktop.resources.get(&resource_address).cloned().ok_or(
                RuntimeError::ApplicationError(ApplicationError::WorktopError(
                    WorktopError::InsufficientBalance,
                )),
            )?);
            let existing_amount = existing_bucket.amount(api)?;

            if existing_amount < amount {
                Err(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ))
            } else if existing_amount == amount {
                // Move
                worktop.resources.swap_remove(&resource_address);
                api.field_write_typed(worktop_handle, &worktop)?;
                api.field_close(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&existing_bucket))
            } else {
                let bucket = existing_bucket.take(amount, api)?;
                api.field_close(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&bucket))
            }
        }
    }

    pub(crate) fn take_non_fungibles<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        let input: WorktopTakeNonFungiblesInput = input
            .into_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let resource_address = input.resource_address;
        let ids = input.ids;

        if ids.is_empty() {
            let bucket = ResourceManager(resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            let worktop_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                WorktopField::Worktop.into(),
                LockFlags::MUTABLE,
            )?;
            let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
            let existing_bucket = Bucket(worktop.resources.get(&resource_address).cloned().ok_or(
                RuntimeError::ApplicationError(ApplicationError::WorktopError(
                    WorktopError::InsufficientBalance,
                )),
            )?);
            let existing_non_fungibles = existing_bucket.non_fungible_local_ids(api)?;

            if !existing_non_fungibles.is_superset(&ids) {
                Err(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ))
            } else if existing_non_fungibles.len() == ids.len() {
                // Move
                worktop = api.field_read_typed(worktop_handle)?;
                worktop.resources.swap_remove(&resource_address);
                api.field_write_typed(worktop_handle, &worktop)?;
                api.field_close(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&existing_bucket))
            } else {
                let bucket = existing_bucket.take_non_fungibles(ids, api)?;
                api.field_close(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&bucket))
            }
        }
    }

    pub(crate) fn take_all<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        let input: WorktopTakeAllInput = input
            .into_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::MUTABLE,
        )?;
        let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        if let Some(bucket) = worktop.resources.swap_remove(&input.resource_address) {
            // Move
            api.field_write_typed(worktop_handle, &worktop)?;
            api.field_close(worktop_handle)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            api.field_close(worktop_handle)?;
            let bucket = ResourceManager(input.resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        }
    }

    pub(crate) fn assert_contains<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        let input: WorktopAssertContainsInput = input
            .into_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::read_only(),
        )?;
        let worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        let amount = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            Bucket(bucket).amount(api)?
        } else {
            Decimal::zero()
        };
        if amount.is_zero() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }
        api.field_close(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn assert_contains_amount<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        let input: WorktopAssertContainsAmountInput = input
            .into_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::read_only(),
        )?;
        let worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        let amount = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            Bucket(bucket).amount(api)?
        } else {
            Decimal::zero()
        };
        if amount < input.amount {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }
        api.field_close(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn assert_contains_non_fungibles<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        let input: WorktopAssertContainsNonFungiblesInput = input
            .into_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::read_only(),
        )?;
        let worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        let ids = if let Some(bucket) = worktop.resources.get(&input.resource_address) {
            let bucket = Bucket(bucket.clone());
            bucket.non_fungible_local_ids(api)?
        } else {
            index_set_new()
        };
        if !ids.is_superset(&input.ids) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }
        api.field_close(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn drain<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        let _input: WorktopDrainInput = input
            .into_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::MUTABLE,
        )?;
        let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        let buckets: Vec<Own> = worktop.resources.values().cloned().collect();
        worktop.resources.clear();
        api.field_write_typed(worktop_handle, &worktop)?;
        api.field_close(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&buckets))
    }
}
