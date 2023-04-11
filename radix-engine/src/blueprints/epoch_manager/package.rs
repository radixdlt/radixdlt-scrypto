use crate::blueprints::epoch_manager::{EpochManagerBlueprint, ValidatorBlueprint};
use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::{event_schema, types::*};
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::{require, AccessRule, FnKey};
use radix_engine_interface::schema::{BlueprintSchema, FunctionSchema, PackageSchema, Receiver};
use resources_tracker_macro::trace_resources;

use super::*;

pub struct EpochManagerNativePackage;

impl EpochManagerNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<EpochManagerSubstate>());
        substates.push(aggregator.add_child_type_and_descendents::<ValidatorSetSubstate>());
        substates.push(aggregator.add_child_type_and_descendents::<ValidatorSetSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            EPOCH_MANAGER_CREATE_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<EpochManagerCreateInput>(),
                output: aggregator.add_child_type_and_descendents::<EpochManagerCreateOutput>(),
                export_name: EPOCH_MANAGER_CREATE_IDENT.to_string(),
            },
        );
        functions.insert(
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator
                    .add_child_type_and_descendents::<EpochManagerGetCurrentEpochInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<EpochManagerGetCurrentEpochOutput>(),
                export_name: EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
            },
        );
        functions.insert(
            EPOCH_MANAGER_SET_EPOCH_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<EpochManagerSetEpochInput>(),
                output: aggregator.add_child_type_and_descendents::<EpochManagerSetEpochOutput>(),
                export_name: EPOCH_MANAGER_SET_EPOCH_IDENT.to_string(),
            },
        );
        functions.insert(
            EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<EpochManagerNextRoundInput>(),
                output: aggregator.add_child_type_and_descendents::<EpochManagerNextRoundOutput>(),
                export_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
            },
        );
        functions.insert(
            EPOCH_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<EpochManagerCreateValidatorInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<EpochManagerCreateValidatorOutput>(),
                export_name: EPOCH_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            },
        );
        functions.insert(
            EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<EpochManagerUpdateValidatorInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<EpochManagerUpdateValidatorOutput>(),
                export_name: EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                RoundChangeEvent,
                EpochChangeEvent
            ]
        };

        let schema = generate_full_schema(aggregator);
        let epoch_manager_schema = BlueprintSchema {
            parent: None,
            schema,
            substates,
            functions,
            virtual_lazy_load_functions: btreemap!(),
            event_schema,
        };

        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<ValidatorSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            VALIDATOR_REGISTER_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<ValidatorRegisterInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorRegisterOutput>(),
                export_name: VALIDATOR_REGISTER_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_UNREGISTER_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<ValidatorUnregisterInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorUnregisterOutput>(),
                export_name: VALIDATOR_UNREGISTER_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_STAKE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<ValidatorStakeInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorStakeOutput>(),
                export_name: VALIDATOR_STAKE_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_UNSTAKE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<ValidatorUnstakeInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorUnstakeOutput>(),
                export_name: VALIDATOR_UNSTAKE_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_CLAIM_XRD_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<ValidatorClaimXrdInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorClaimXrdOutput>(),
                export_name: VALIDATOR_CLAIM_XRD_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_UPDATE_KEY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<ValidatorUpdateKeyInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorUpdateKeyOutput>(),
                export_name: VALIDATOR_UPDATE_KEY_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<ValidatorUpdateAcceptDelegatedStakeInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ValidatorUpdateAcceptDelegatedStakeOutput>(),
                export_name: VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                RegisterValidatorEvent,
                UnregisterValidatorEvent,
                StakeEvent,
                UnstakeEvent,
                ClaimXrdEvent,
                UpdateAcceptingStakeDelegationStateEvent
            ]
        };

        let schema = generate_full_schema(aggregator);
        let validator_schema = BlueprintSchema {
            parent: None,
            schema,
            substates,
            functions,
            virtual_lazy_load_functions: btreemap!(),
            event_schema,
        };

        PackageSchema {
            blueprints: btreemap!(
                EPOCH_MANAGER_BLUEPRINT.to_string() => epoch_manager_schema,
                VALIDATOR_BLUEPRINT.to_string() => validator_schema
            ),
        }
    }

    pub fn package_access_rules() -> BTreeMap<FnKey, AccessRule> {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            FnKey::new(
                EPOCH_MANAGER_BLUEPRINT.to_string(),
                EPOCH_MANAGER_CREATE_IDENT.to_string(),
            ),
            rule!(require(AuthAddresses::system_role())),
        );
        access_rules
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            EPOCH_MANAGER_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: EpochManagerCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = EpochManagerBlueprint::create(
                    input.validator_owner_token,
                    input.component_address,
                    input.validator_set,
                    input.initial_epoch,
                    input.rounds_per_epoch,
                    input.num_unstake_epochs,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: EpochManagerGetCurrentEpochInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = EpochManagerBlueprint::get_current_epoch(receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_SET_EPOCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: EpochManagerSetEpochInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = EpochManagerBlueprint::set_epoch(receiver, input.epoch, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_NEXT_ROUND_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: EpochManagerNextRoundInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = EpochManagerBlueprint::next_round(receiver, input.round, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_CREATE_VALIDATOR_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: EpochManagerCreateValidatorInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = EpochManagerBlueprint::create_validator(receiver, input.key, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: EpochManagerUpdateValidatorInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = EpochManagerBlueprint::update_validator(
                    receiver,
                    input.validator_address,
                    input.update,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_REGISTER_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::register(receiver, input, api)
            }
            VALIDATOR_UNREGISTER_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::unregister(receiver, input, api)
            }
            VALIDATOR_STAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::stake(receiver, input, api)
            }
            VALIDATOR_UNSTAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::unstake(receiver, input, api)
            }
            VALIDATOR_CLAIM_XRD_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::claim_xrd(receiver, input, api)
            }
            VALIDATOR_UPDATE_KEY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::update_key(receiver, input, api)
            }
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

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
