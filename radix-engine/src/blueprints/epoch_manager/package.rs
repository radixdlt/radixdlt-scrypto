use crate::blueprints::epoch_manager::{EpochManagerBlueprint, ValidatorBlueprint};
use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::{AccessRule, require};
use radix_engine_interface::data::ScryptoValue;

pub struct EpochManagerNativePackage;

impl EpochManagerNativePackage {
    pub fn package_access_rules() -> BTreeMap<(String, String), AccessRule> {
        let mut access_rules = BTreeMap::new();
        access_rules.insert((EPOCH_MANAGER_BLUEPRINT.to_string(), EPOCH_MANAGER_CREATE_IDENT.to_string()), rule!(require(AuthAddresses::system_role())));
        access_rules
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        match export_name {
            EPOCH_MANAGER_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                EpochManagerBlueprint::create(input, api)
            }
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::get_current_epoch(receiver, input, api)
            }
            EPOCH_MANAGER_SET_EPOCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::set_epoch(receiver, input, api)
            }
            EPOCH_MANAGER_NEXT_ROUND_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::next_round(receiver, input, api)
            }
            EPOCH_MANAGER_CREATE_VALIDATOR_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::create_validator(receiver, input, api)
            }
            EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::update_validator(receiver, input, api)
            }
            VALIDATOR_REGISTER_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::register(receiver, input, api)
            }
            VALIDATOR_UNREGISTER_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::unregister(receiver, input, api)
            }
            VALIDATOR_STAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::stake(receiver, input, api)
            }
            VALIDATOR_UNSTAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::unstake(receiver, input, api)
            }
            VALIDATOR_CLAIM_XRD_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::claim_xrd(receiver, input, api)
            }
            VALIDATOR_UPDATE_KEY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::update_key(receiver, input, api)
            }
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::update_accept_delegated_stake(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
