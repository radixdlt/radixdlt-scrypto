use crate::blueprints::resource::*;
use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE, FIXED_MEDIUM_FEE};
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

pub struct ResourceManagerNativePackage;

impl ResourceManagerNativePackage {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_non_fungible(input, api)
            }
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_non_fungible_with_address(input, api)
            }
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_non_fungible_with_initial_supply(input, api)
            }
            RESOURCE_MANAGER_CREATE_UUID_NON_FUNGIBLE_WITH_INITIAL_SUPPLY => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_uuid_non_fungible_with_initial_supply(input, api)
            }
            RESOURCE_MANAGER_CREATE_FUNGIBLE_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_fungible(input, api)
            }
            RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_fungible_with_initial_supply(input, api)
            }
            RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_fungible_with_initial_supply_and_address(
                    input, api,
                )
            }
            RESOURCE_MANAGER_BURN_BUCKET_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::burn_bucket(input, api)
            }
            RESOURCE_MANAGER_MINT_NON_FUNGIBLE => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::mint_non_fungible(receiver, input, api)
            }
            RESOURCE_MANAGER_MINT_UUID_NON_FUNGIBLE => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::mint_uuid_non_fungible(receiver, input, api)
            }
            RESOURCE_MANAGER_MINT_FUNGIBLE => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::mint_fungible(receiver, input, api)
            }
            RESOURCE_MANAGER_BURN_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::burn(receiver, input, api)
            }
            RESOURCE_MANAGER_CREATE_BUCKET_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::create_bucket(receiver, input, api)
            }
            RESOURCE_MANAGER_CREATE_VAULT_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::create_vault(receiver, input, api)
            }
            RESOURCE_MANAGER_UPDATE_NON_FUNGIBLE_DATA_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::update_non_fungible_data(receiver, input, api)
            }
            RESOURCE_MANAGER_NON_FUNGIBLE_EXISTS_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::non_fungible_exists(receiver, input, api)
            }
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::get_resource_type(receiver, input, api)
            }
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::get_total_supply(receiver, input, api)
            }
            RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::get_non_fungible(receiver, input, api)
            }
            VAULT_LOCK_FEE_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::lock_fee(receiver, input, api)
            }
            VAULT_TAKE_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::take(receiver, input, api)
            }
            VAULT_TAKE_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::take_non_fungibles(receiver, input, api)
            }
            VAULT_RECALL_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::recall(receiver, input, api)
            }
            VAULT_RECALL_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::recall_non_fungibles(receiver, input, api)
            }
            VAULT_PUT_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::put(receiver, input, api)
            }
            VAULT_GET_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::get_amount(receiver, input, api)
            }
            VAULT_GET_RESOURCE_ADDRESS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::get_resource_address(receiver, input, api)
            }
            VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::get_non_fungible_local_ids(receiver, input, api)
            }
            VAULT_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::create_proof(receiver, input, api)
            }
            VAULT_CREATE_PROOF_BY_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::create_proof_by_amount(receiver, input, api)
            }
            VAULT_CREATE_PROOF_BY_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::create_proof_by_ids(receiver, input, api)
            }
            VAULT_LOCK_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::lock_amount(receiver, input, api)
            }
            VAULT_LOCK_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::lock_non_fungibles(receiver, input, api)
            }
            VAULT_UNLOCK_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::unlock_amount(receiver, input, api)
            }
            VAULT_UNLOCK_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::unlock_non_fungibles(receiver, input, api)
            }
            PROOF_CLONE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ProofBlueprint::clone(receiver, input, api)
            }
            PROOF_GET_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ProofBlueprint::get_amount(receiver, input, api)
            }
            PROOF_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ProofBlueprint::get_non_fungible_local_ids(receiver, input, api)
            }
            PROOF_GET_RESOURCE_ADDRESS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ProofBlueprint::get_resource_address(receiver, input, api)
            }
            BUCKET_DROP_EMPTY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                BucketBlueprint::drop_empty(input, api)
            }
            BUCKET_PUT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::put(receiver, input, api)
            }
            BUCKET_TAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::take(receiver, input, api)
            }
            BUCKET_TAKE_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::take_non_fungibles(receiver, input, api)
            }
            BUCKET_GET_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::get_amount(receiver, input, api)
            }
            BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::get_non_fungible_local_ids(receiver, input, api)
            }
            BUCKET_GET_RESOURCE_ADDRESS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::get_resource_address(receiver, input, api)
            }
            BUCKET_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::create_proof(receiver, input, api)
            }
            BUCKET_LOCK_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::lock_amount(receiver, input, api)
            }
            BUCKET_LOCK_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::lock_non_fungibles(receiver, input, api)
            }
            BUCKET_UNLOCK_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::unlock_amount(receiver, input, api)
            }
            BUCKET_UNLOCK_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                BucketBlueprint::unlock_non_fungibles(receiver, input, api)
            }
            WORKTOP_DROP_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                WorktopBlueprint::drop(input, api)
            }
            WORKTOP_PUT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                WorktopBlueprint::put(receiver, input, api)
            }
            WORKTOP_TAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                WorktopBlueprint::take(receiver, input, api)
            }
            WORKTOP_TAKE_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                WorktopBlueprint::take_non_fungibles(receiver, input, api)
            }
            WORKTOP_TAKE_ALL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                WorktopBlueprint::take_all(receiver, input, api)
            }
            WORKTOP_ASSERT_CONTAINS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                WorktopBlueprint::assert_contains(receiver, input, api)
            }
            WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                WorktopBlueprint::assert_contains_amount(receiver, input, api)
            }
            WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                WorktopBlueprint::assert_contains_non_fungibles(receiver, input, api)
            }
            WORKTOP_DRAIN_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                WorktopBlueprint::drain(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
