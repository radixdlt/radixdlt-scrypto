use crate::blueprints::epoch_manager::{EpochManagerBlueprint, ValidatorBlueprint};
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
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
        substates.push(aggregator.add_child_type_and_descendents::<EpochManagerConfigSubstate>());
        substates.push(aggregator.add_child_type_and_descendents::<EpochManagerSubstate>());
        substates.push(aggregator.add_child_type_and_descendents::<CurrentValidatorSetSubstate>());
        substates.push(aggregator.add_child_type_and_descendents::<SecondaryIndexSubstate>());

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
            EPOCH_MANAGER_START_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<EpochManagerStartInput>(),
                output: aggregator.add_child_type_and_descendents::<EpochManagerStartOutput>(),
                export_name: EPOCH_MANAGER_START_IDENT.to_string(),
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

        let event_schema = event_schema! {
            aggregator,
            [
                RoundChangeEvent,
                EpochChangeEvent
            ]
        };

        let schema = generate_full_schema(aggregator);
        let epoch_manager_schema = BlueprintSchema {
            outer_blueprint: None,
            schema,
            substates: substates,
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
            outer_blueprint: Some(EPOCH_MANAGER_BLUEPRINT.to_string()),
            schema,
            substates: substates,
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
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        match export_name {
            EPOCH_MANAGER_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: EpochManagerCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = EpochManagerBlueprint::create(
                    input.validator_owner_token,
                    input.component_address,
                    input.initial_epoch,
                    input.max_validators,
                    input.rounds_per_epoch,
                    input.num_unstake_epochs,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: EpochManagerGetCurrentEpochInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = EpochManagerBlueprint::get_current_epoch(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_SET_EPOCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: EpochManagerSetEpochInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = EpochManagerBlueprint::set_epoch(input.epoch, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_START_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: EpochManagerStartInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = EpochManagerBlueprint::start(receiver, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_NEXT_ROUND_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: EpochManagerNextRoundInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = EpochManagerBlueprint::next_round(input.round, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            EPOCH_MANAGER_CREATE_VALIDATOR_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: EpochManagerCreateValidatorInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = EpochManagerBlueprint::create_validator(input.key, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_REGISTER_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ValidatorRegisterInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::register(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UNREGISTER_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ValidatorUnregisterInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::unregister(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_STAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ValidatorStakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::stake(input.stake, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UNSTAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ValidatorUnstakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::unstake(input.lp_tokens, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_CLAIM_XRD_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ValidatorClaimXrdInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::claim_xrd(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UPDATE_KEY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ValidatorUpdateKeyInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::update_key(input.key, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: ValidatorUpdateAcceptDelegatedStakeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::update_accept_delegated_stake(
                    receiver,
                    input.accept_delegated_stake,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
