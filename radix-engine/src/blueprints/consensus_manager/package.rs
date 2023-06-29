use crate::blueprints::consensus_manager::{ConsensusManagerBlueprint, ValidatorBlueprint};
use crate::errors::{ApplicationError, RuntimeError};
use crate::kernel::kernel_api::KernelNodeApi;
use crate::{event_schema, roles_template, types::*};
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::blueprints::resource::require;
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintFunctionsSchemaInit, BlueprintSchemaInit,
    BlueprintSortedIndexSchema, BlueprintStateSchemaInit, FieldSchema, FunctionSchemaInit,
    ReceiverInfo, TypeRef,
};

use super::*;

pub const VALIDATOR_ROLE: &str = "validator";

pub struct ConsensusManagerNativePackage;

impl ConsensusManagerNativePackage {
    pub fn definition() -> PackageDefinition {
        let consensus_manager_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();

            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<ConsensusManagerConfigSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<ConsensusManagerSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<ValidatorRewardsSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<CurrentValidatorSetSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<CurrentProposalStatisticSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<ProposerMinuteTimestampSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<ProposerMilliTimestampSubstate>(),
            ));

            let mut collections = Vec::new();
            collections.push(BlueprintCollectionSchema::SortedIndex(
                BlueprintSortedIndexSchema {},
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                CONSENSUS_MANAGER_CREATE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ConsensusManagerCreateInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ConsensusManagerCreateOutput>(),
                    ),
                    export: CONSENSUS_MANAGER_CREATE_IDENT.to_string(),
                },
            );
            functions.insert(
                CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ConsensusManagerGetCurrentEpochInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ConsensusManagerGetCurrentEpochOutput>()),
                    export: CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
                },
            );
            functions.insert(
                CONSENSUS_MANAGER_START_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ConsensusManagerStartInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ConsensusManagerStartOutput>(),
                    ),
                    export: CONSENSUS_MANAGER_START_IDENT.to_string(),
                },
            );
            functions.insert(
                CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ConsensusManagerGetCurrentTimeInput>(
                            ),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ConsensusManagerGetCurrentTimeOutput>(
                            ),
                    ),
                    export: CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT.to_string(),
                },
            );
            functions.insert(
                CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ConsensusManagerCompareCurrentTimeInput>(
                        )),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ConsensusManagerCompareCurrentTimeOutput>(
                        )),
                    export: CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT.to_string(),
                },
            );
            functions.insert(
                CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ConsensusManagerNextRoundInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ConsensusManagerNextRoundOutput>(),
                    ),
                    export: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
                },
            );
            functions.insert(
                CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ConsensusManagerCreateValidatorInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ConsensusManagerCreateValidatorOutput>()),
                    export: CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
                },
            );

            let event_schema = event_schema! {
                aggregator,
                [
                    RoundChangeEvent,
                    EpochChangeEvent
                ]
            };

            let consensus_manager_schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                feature_set: btreeset!(),
                dependencies: btreeset!(
                    RADIX_TOKEN.into(),
                    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                    SYSTEM_TRANSACTION_BADGE.into(),
                    VALIDATOR_OWNER_BADGE.into(),
                ),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema: consensus_manager_schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: event_schema,
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AccessRules(btreemap!(
                        CONSENSUS_MANAGER_CREATE_IDENT.to_string() => rule!(require(AuthAddresses::system_role())),
                    )),
                    method_auth: MethodAuthTemplate::StaticRoles(roles_template!(
                        roles {
                            VALIDATOR_ROLE;
                        },
                        methods {
                            CONSENSUS_MANAGER_START_IDENT => []; // Genesis is able to call this by skipping auth
                            CONSENSUS_MANAGER_NEXT_ROUND_IDENT => [VALIDATOR_ROLE];

                            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT => MethodAccessibility::Public;
                            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT => MethodAccessibility::Public;
                            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT => MethodAccessibility::Public;
                            CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT => MethodAccessibility::Public;
                        }
                    )),
                },
            }
        };

        let validator_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<ValidatorSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<ValidatorProtocolUpdateReadinessSignalSubstate>(),
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                VALIDATOR_REGISTER_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorRegisterInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorRegisterOutput>(),
                    ),
                    export: VALIDATOR_REGISTER_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_UNREGISTER_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorUnregisterInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorUnregisterOutput>(),
                    ),
                    export: VALIDATOR_UNREGISTER_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_STAKE_AS_OWNER_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorStakeAsOwnerInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorStakeAsOwnerOutput>(),
                    ),
                    export: VALIDATOR_STAKE_AS_OWNER_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_STAKE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorStakeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorStakeOutput>(),
                    ),
                    export: VALIDATOR_STAKE_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_UNSTAKE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorUnstakeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorUnstakeOutput>(),
                    ),
                    export: VALIDATOR_UNSTAKE_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_CLAIM_XRD_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorClaimXrdInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorClaimXrdOutput>(),
                    ),
                    export: VALIDATOR_CLAIM_XRD_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_UPDATE_KEY_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorUpdateKeyInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorUpdateKeyOutput>(),
                    ),
                    export: VALIDATOR_UPDATE_KEY_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_UPDATE_FEE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorUpdateFeeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorUpdateFeeOutput>(),
                    ),
                    export: VALIDATOR_UPDATE_FEE_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ValidatorUpdateAcceptDelegatedStakeInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ValidatorUpdateAcceptDelegatedStakeOutput>()),
                    export: VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ValidatorAcceptsDelegatedStakeInput>(
                            ),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ValidatorAcceptsDelegatedStakeOutput>(
                            ),
                    ),
                    export: VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ValidatorSignalProtocolUpdateReadinessInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ValidatorSignalProtocolUpdateReadinessOutput>()),
                    export: VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ValidatorLockOwnerStakeUnitsInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ValidatorLockOwnerStakeUnitsOutput>(),
                    ),
                    export: VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ValidatorStartUnlockOwnerStakeUnitsInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ValidatorStartUnlockOwnerStakeUnitsOutput>()),
                    export: VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ValidatorFinishUnlockOwnerStakeUnitsInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ValidatorFinishUnlockOwnerStakeUnitsOutput>()),
                    export: VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_APPLY_EMISSION_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorApplyEmissionInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorApplyEmissionOutput>(),
                    ),
                    export: VALIDATOR_APPLY_EMISSION_IDENT.to_string(),
                },
            );
            functions.insert(
                VALIDATOR_APPLY_REWARD_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorApplyRewardInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ValidatorApplyRewardOutput>(),
                    ),
                    export: VALIDATOR_APPLY_REWARD_IDENT.to_string(),
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
                    ProtocolUpdateReadinessSignalEvent,
                    UpdateAcceptingStakeDelegationStateEvent,
                    ValidatorEmissionAppliedEvent,
                    ValidatorRewardAppliedEvent
                ]
            };

            let schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                blueprint_type: BlueprintType::Inner {
                    outer_blueprint: CONSENSUS_MANAGER_BLUEPRINT.to_string(),
                },
                feature_set: btreeset!(),
                dependencies: btreeset!(),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: event_schema,
                    functions: BlueprintFunctionsSchemaInit {
                        virtual_lazy_load_functions: btreemap!(),
                        functions,
                    },
                },
                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoles(roles_template! {
                        methods {
                            VALIDATOR_UNSTAKE_IDENT => MethodAccessibility::Public;
                            VALIDATOR_CLAIM_XRD_IDENT => MethodAccessibility::Public;
                            VALIDATOR_STAKE_IDENT => MethodAccessibility::Public;
                            VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT => MethodAccessibility::Public;
                            VALIDATOR_STAKE_AS_OWNER_IDENT => [OWNER_ROLE];
                            VALIDATOR_REGISTER_IDENT => [OWNER_ROLE];
                            VALIDATOR_UNREGISTER_IDENT => [OWNER_ROLE];
                            VALIDATOR_UPDATE_KEY_IDENT => [OWNER_ROLE];
                            VALIDATOR_UPDATE_FEE_IDENT => [OWNER_ROLE];
                            VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT => [OWNER_ROLE];
                            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT => [OWNER_ROLE];
                            VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT => [OWNER_ROLE];
                            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => [OWNER_ROLE];
                            VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS => [OWNER_ROLE];
                            VALIDATOR_APPLY_EMISSION_IDENT => MethodAccessibility::OuterObjectOnly;
                            VALIDATOR_APPLY_REWARD_IDENT => MethodAccessibility::OuterObjectOnly;
                        }
                    }),
                },
            }
        };

        let blueprints = btreemap!(
            CONSENSUS_MANAGER_BLUEPRINT.to_string() => consensus_manager_blueprint,
            VALIDATOR_BLUEPRINT.to_string() => validator_blueprint,
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        match export_name {
            CONSENSUS_MANAGER_CREATE_IDENT => {
                let input: ConsensusManagerCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::create(
                    input.validator_owner_token_address,
                    input.component_address,
                    input.initial_epoch,
                    input.initial_config,
                    input.initial_time_ms,
                    input.initial_current_leader,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT => {
                let _input: ConsensusManagerGetCurrentEpochInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;

                let rtn = ConsensusManagerBlueprint::get_current_epoch(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_START_IDENT => {
                let _input: ConsensusManagerStartInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::start(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT => {
                let input: ConsensusManagerGetCurrentTimeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::get_current_time(input.precision, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT => {
                let input: ConsensusManagerCompareCurrentTimeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                let input: ConsensusManagerNextRoundInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                let input: ConsensusManagerCreateValidatorInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ConsensusManagerBlueprint::create_validator(
                    input.key,
                    input.fee_factor,
                    input.xrd_payment,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_REGISTER_IDENT => {
                let _input: ValidatorRegisterInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::register(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UNREGISTER_IDENT => {
                let _input: ValidatorUnregisterInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::unregister(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_STAKE_AS_OWNER_IDENT => {
                let input: ValidatorStakeAsOwnerInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::stake_as_owner(input.stake, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_STAKE_IDENT => {
                let input: ValidatorStakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::stake(input.stake, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UNSTAKE_IDENT => {
                let input: ValidatorUnstakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::unstake(input.stake_unit_bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_CLAIM_XRD_IDENT => {
                let input: ValidatorClaimXrdInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::claim_xrd(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UPDATE_KEY_IDENT => {
                let input: ValidatorUpdateKeyInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::update_key(input.key, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UPDATE_FEE_IDENT => {
                let input: ValidatorUpdateFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::update_fee(input.new_fee_factor, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => {
                let input: ValidatorUpdateAcceptDelegatedStakeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::update_accept_delegated_stake(
                    input.accept_delegated_stake,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT => {
                let _: ValidatorAcceptsDelegatedStakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::accepts_delegated_stake(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS => {
                let input: ValidatorSignalProtocolUpdateReadinessInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::signal_protocol_update_readiness(input.vote, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT => {
                let input: ValidatorLockOwnerStakeUnitsInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::lock_owner_stake_units(input.stake_unit_bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT => {
                let input: ValidatorStartUnlockOwnerStakeUnitsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::start_unlock_owner_stake_units(
                    input.requested_stake_unit_amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT => {
                let _input: ValidatorFinishUnlockOwnerStakeUnitsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::finish_unlock_owner_stake_units(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_APPLY_EMISSION_IDENT => {
                let input: ValidatorApplyEmissionInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
            VALIDATOR_APPLY_REWARD_IDENT => {
                let input: ValidatorApplyRewardInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::apply_reward(input.xrd_bucket, input.epoch, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
