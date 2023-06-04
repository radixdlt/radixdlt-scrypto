use crate::blueprints::consensus_manager::{ConsensusManagerBlueprint, ValidatorBlueprint};
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::{event_schema, method_auth_template, types::*};
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::node_modules::metadata::{
    METADATA_GET_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_IDENT,
};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::package::{
    BlueprintSetup, BlueprintTemplate, PackageSetup,
};
use radix_engine_interface::blueprints::resource::require;
use radix_engine_interface::schema::{BlueprintCollectionSchema, BlueprintSchema, BlueprintSortedIndexSchema, ExportNameMapping, FunctionSchema, ReceiverInfo, SchemaMethodKey, SchemaMethodPermission};
use resources_tracker_macro::trace_resources;

use super::*;

pub const VALIDATOR_ROLE: &str = "validator";
pub const START_ROLE: &str = "start";

pub const VALIDATOR_APPLY_EMISSION_AUTHORITY: &str = "apply_emission";

pub struct ConsensusManagerNativePackage;

impl ConsensusManagerNativePackage {
    pub fn definition() -> PackageSetup {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(aggregator.add_child_type_and_descendents::<ConsensusManagerConfigSubstate>());
        fields.push(aggregator.add_child_type_and_descendents::<ConsensusManagerSubstate>());
        fields.push(aggregator.add_child_type_and_descendents::<CurrentValidatorSetSubstate>());
        fields
            .push(aggregator.add_child_type_and_descendents::<CurrentProposalStatisticSubstate>());
        fields.push(aggregator.add_child_type_and_descendents::<ProposerMinuteTimestampSubstate>());
        fields.push(aggregator.add_child_type_and_descendents::<ProposerMilliTimestampSubstate>());

        let mut collections = Vec::new();
        collections.push(BlueprintCollectionSchema::SortedIndex(
            BlueprintSortedIndexSchema {},
        ));

        let mut functions = BTreeMap::new();
        functions.insert(
            CONSENSUS_MANAGER_CREATE_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<ConsensusManagerCreateInput>(),
                output: aggregator.add_child_type_and_descendents::<ConsensusManagerCreateOutput>(),
                export: ExportNameMapping::normal(CONSENSUS_MANAGER_CREATE_IDENT),
            },
        );
        functions.insert(
            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerGetCurrentEpochInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerGetCurrentEpochOutput>(),
                export: ExportNameMapping::normal(CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT),
            },
        );
        functions.insert(
            CONSENSUS_MANAGER_START_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<ConsensusManagerStartInput>(),
                output: aggregator.add_child_type_and_descendents::<ConsensusManagerStartOutput>(),
                export: ExportNameMapping::normal(CONSENSUS_MANAGER_START_IDENT),
            },
        );
        functions.insert(
            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerGetCurrentTimeInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerGetCurrentTimeOutput>(),
                export: ExportNameMapping::normal(CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT),
            },
        );
        functions.insert(
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerCompareCurrentTimeInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerCompareCurrentTimeOutput>(),
                export: ExportNameMapping::normal(CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT),
            },
        );
        functions.insert(
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerNextRoundInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerNextRoundOutput>(),
                export: ExportNameMapping::normal(CONSENSUS_MANAGER_NEXT_ROUND_IDENT),
            },
        );
        functions.insert(
            CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerCreateValidatorInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ConsensusManagerCreateValidatorOutput>(),
                export: ExportNameMapping::normal(CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT),
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
        let consensus_manager_schema = BlueprintSchema {
            outer_blueprint: None,
            schema,
            fields,
            collections,
            functions,
            virtual_lazy_load_functions: btreemap!(),
            event_schema,
            dependencies: btreeset!(
                RADIX_TOKEN.into(),
                PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                SYSTEM_TRANSACTION_BADGE.into(),
                VALIDATOR_OWNER_BADGE.into(),
            ),
        };

        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(aggregator.add_child_type_and_descendents::<ValidatorSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            VALIDATOR_REGISTER_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<ValidatorRegisterInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorRegisterOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_REGISTER_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_UNREGISTER_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<ValidatorUnregisterInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorUnregisterOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_UNREGISTER_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_STAKE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<ValidatorStakeInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorStakeOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_STAKE_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_UNSTAKE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<ValidatorUnstakeInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorUnstakeOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_UNSTAKE_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_CLAIM_XRD_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<ValidatorClaimXrdInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorClaimXrdOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_CLAIM_XRD_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_UPDATE_KEY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<ValidatorUpdateKeyInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorUpdateKeyOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_UPDATE_KEY_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_UPDATE_FEE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<ValidatorUpdateFeeInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorUpdateFeeOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_UPDATE_FEE_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<ValidatorUpdateAcceptDelegatedStakeInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ValidatorUpdateAcceptDelegatedStakeOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<ValidatorLockOwnerStakeUnitsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ValidatorLockOwnerStakeUnitsOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<ValidatorStartUnlockOwnerStakeUnitsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ValidatorStartUnlockOwnerStakeUnitsOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<ValidatorFinishUnlockOwnerStakeUnitsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ValidatorFinishUnlockOwnerStakeUnitsOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT),
            },
        );
        functions.insert(
            VALIDATOR_APPLY_EMISSION_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<ValidatorApplyEmissionInput>(),
                output: aggregator.add_child_type_and_descendents::<ValidatorApplyEmissionOutput>(),
                export: ExportNameMapping::normal(VALIDATOR_APPLY_EMISSION_IDENT),
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
                UpdateAcceptingStakeDelegationStateEvent,
                ValidatorEmissionAppliedEvent
            ]
        };

        let schema = generate_full_schema(aggregator);

        let validator_schema = BlueprintSchema {
            outer_blueprint: Some(CONSENSUS_MANAGER_BLUEPRINT.to_string()),
            schema,
            fields,
            collections: vec![],
            functions,
            virtual_lazy_load_functions: btreemap!(),
            event_schema,
            dependencies: btreeset!(),
        };

        let blueprints = btreemap!(
            CONSENSUS_MANAGER_BLUEPRINT.to_string() => BlueprintSetup {
                schema: consensus_manager_schema,
                function_auth: btreemap!(
                    CONSENSUS_MANAGER_CREATE_IDENT.to_string() => rule!(require(AuthAddresses::system_role())),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: method_auth_template!(
                        SchemaMethodKey::main(CONSENSUS_MANAGER_START_IDENT) => [START_ROLE];
                        SchemaMethodKey::main(CONSENSUS_MANAGER_NEXT_ROUND_IDENT) => [VALIDATOR_ROLE];

                        SchemaMethodKey::main(CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT) => SchemaMethodPermission::Public;
                    ),
                    outer_method_auth_template: btreemap!(),
                },
            },
            VALIDATOR_BLUEPRINT.to_string() => BlueprintSetup {
                schema: validator_schema,
                function_auth: btreemap!(),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: method_auth_template! {
                        SchemaMethodKey::metadata(METADATA_SET_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(VALIDATOR_UNSTAKE_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VALIDATOR_CLAIM_XRD_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VALIDATOR_STAKE_IDENT) => [STAKE_ROLE];
                        SchemaMethodKey::main(VALIDATOR_REGISTER_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(VALIDATOR_UNREGISTER_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(VALIDATOR_UPDATE_KEY_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(VALIDATOR_UPDATE_FEE_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(VALIDATOR_APPLY_EMISSION_IDENT) => [VALIDATOR_APPLY_EMISSION_AUTHORITY];
                    },
                    outer_method_auth_template: btreemap!(),
                },
            }
        );

        PackageSetup { blueprints }
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
            CONSENSUS_MANAGER_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: ConsensusManagerCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::create(
                    input.validator_owner_token,
                    input.component_address,
                    input.initial_epoch,
                    input.initial_config,
                    input.initial_time_ms,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ConsensusManagerGetCurrentEpochInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn = ConsensusManagerBlueprint::get_current_epoch(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_START_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: ConsensusManagerStartInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::start(receiver, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ConsensusManagerGetCurrentTimeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::get_current_time(input.precision, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ConsensusManagerCompareCurrentTimeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = ConsensusManagerBlueprint::compare_current_time(
                    input.instant,
                    input.precision,
                    input.operator,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ConsensusManagerNextRoundInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::next_round(
                    input.round,
                    input.proposer_timestamp_ms,
                    input.leader_proposal_history,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ConsensusManagerCreateValidatorInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = ConsensusManagerBlueprint::create_validator(input.key, api)?;

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
                let rtn = ValidatorBlueprint::unstake(input.stake_unit_bucket, api)?;
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
            VALIDATOR_UPDATE_FEE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ValidatorUpdateFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::update_fee(input.new_fee_factor, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ValidatorUpdateAcceptDelegatedStakeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::update_accept_delegated_stake(
                    input.accept_delegated_stake,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ValidatorLockOwnerStakeUnitsInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::lock_owner_stake_units(input.stake_unit_bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ValidatorStartUnlockOwnerStakeUnitsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::start_unlock_owner_stake_units(
                    input.requested_stake_unit_amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ValidatorFinishUnlockOwnerStakeUnitsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::finish_unlock_owner_stake_units(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_APPLY_EMISSION_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ValidatorApplyEmissionInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::apply_emission(
                    input.xrd_bucket,
                    input.epoch,
                    input.proposals_made,
                    input.proposals_missed,
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
