use super::{EpochChangeEvent, RoundChangeEvent, ValidatorCreator, ValidatorOwnerBadgeData};
use crate::blueprints::consensus_manager::VALIDATOR_ROLE;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::object_api::ModuleId;
use radix_engine_interface::api::{
    AttachedModuleId, CollectionIndex, FieldValue, SystemApi, ACTOR_STATE_SELF,
};
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::package::BlueprintDefinitionInit;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::UncheckedUrl;
use radix_engine_interface::{metadata_init, mint_roles, rule};
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_native_sdk::resource::NativeVault;
use radix_native_sdk::resource::{NativeBucket, ResourceManager};
use radix_native_sdk::runtime::Runtime;

const MILLIS_IN_SECOND: i64 = 1000;
const SECONDS_IN_MINUTE: i64 = 60;
const MILLIS_IN_MINUTE: i64 = MILLIS_IN_SECOND * SECONDS_IN_MINUTE;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ConsensusManagerConfigSubstate {
    pub config: ConsensusManagerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ConsensusManagerSubstate {
    /// Whether the consensus process has started
    pub started: bool,
    /// The current epoch.
    pub epoch: Epoch,
    /// The effective start-time of the epoch.
    /// This is used to calculate the effective duration, for the purpose of calculating
    /// when to change epoch. This will typically be close to the `actual_epoch_start_milli`
    /// but may differ slightly as it attempts to avoid minor systematic drift in the epoch
    /// start time.
    pub effective_epoch_start_milli: i64,
    /// The actual start-time of the epoch.
    /// This is just saved as a sanity-check for checking divergence between actual and effective.
    pub actual_epoch_start_milli: i64,
    /// The current round in the epoch.
    pub round: Round,
    /// The current leader - this is used for knowing who was the validator for the following
    /// round of transactions
    pub current_leader: Option<ValidatorIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, ScryptoSbor)]
pub struct Validator {
    pub key: Secp256k1PublicKey,
    pub stake: Decimal,
}

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct ValidatorRewardsSubstate {
    pub proposer_rewards: IndexMap<ValidatorIndex, Decimal>,
    pub rewards_vault: Vault,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct CurrentValidatorSetSubstate {
    pub validator_set: ActiveValidatorSet,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct ActiveValidatorSet {
    /// The validators in the set, ordered by stake descending.
    pub validators_by_stake_desc: IndexMap<ComponentAddress, Validator>,
}

impl ActiveValidatorSet {
    pub fn get_by_index(&self, index: ValidatorIndex) -> Option<(&ComponentAddress, &Validator)> {
        self.validators_by_stake_desc.get_index(index as usize)
    }

    pub fn get_by_address(&self, address: &ComponentAddress) -> Option<&Validator> {
        self.validators_by_stake_desc.get(address)
    }

    /// Note for performance - this is calculated by iterating over the whole validator set.
    pub fn get_by_public_key(
        &self,
        public_key: &Secp256k1PublicKey,
    ) -> Option<(&ComponentAddress, &Validator)> {
        self.validators_by_stake_desc
            .iter()
            .find(|(_, validator)| &validator.key == public_key)
    }

    /// Note for performance - this is calculated by iterating over the whole validator set.
    pub fn total_active_stake_xrd(&self) -> Result<Decimal, RuntimeError> {
        let mut sum = Decimal::ZERO;
        for v in self
            .validators_by_stake_desc
            .iter()
            .map(|(_, validator)| validator.stake)
        {
            sum = sum.checked_add(v).ok_or(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::UnexpectedDecimalComputationError,
                ),
            ))?;
        }
        Ok(sum)
    }

    pub fn validator_count(&self) -> usize {
        self.validators_by_stake_desc.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct ProposerMilliTimestampSubstate {
    /// A number of millis elapsed since epoch (i.e. a classic "epoch millis" timestamp).
    /// A signed number is traditionally used (for reasons like representing instants before A.D.
    /// 1970, which may not even apply in our case).
    pub epoch_milli: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct ProposerMinuteTimestampSubstate {
    /// A number of full minutes elapsed since epoch.
    /// A signed number is used for the same reasons as in [`ProposerMilliTimestampSubstate`], and
    /// gives us time until A.D. 5772.
    pub epoch_minute: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct CurrentProposalStatisticSubstate {
    /// A proposal statistic of each validator from the current validator set, in the iteration
    /// order of [`CurrentValidatorSetSubstate.validator_set`].
    pub validator_statistics: Vec<ProposalStatistic>,
}

impl CurrentProposalStatisticSubstate {
    /// Gets a mutable reference to a proposal statistic tracker of an individual validator.
    pub fn get_mut_proposal_statistic(
        &mut self,
        validator_index: ValidatorIndex,
    ) -> Result<&mut ProposalStatistic, RuntimeError> {
        let validator_count = self.validator_statistics.len();
        self.validator_statistics
            .get_mut(validator_index as usize)
            .ok_or_else(|| {
                RuntimeError::ApplicationError(ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::InvalidValidatorIndex {
                        index: validator_index,
                        count: validator_count,
                    },
                ))
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor)]
pub struct ProposalStatistic {
    /// A counter of successful proposals made by a specific validator.
    pub made: u64,
    /// A counter of missed proposals (caused both by gap rounds or fallback rounds).
    pub missed: u64,
}

impl ProposalStatistic {
    /// A ratio of successful to total proposals.
    /// There is a special case of a validator which did not have a chance of leading even a single
    /// round of consensus - currently we assume they should not be punished (i.e. we return `1.0`).
    pub fn success_ratio(&self) -> Result<Decimal, RuntimeError> {
        let total = self.made + self.missed;
        if total == 0 {
            return Ok(Decimal::one());
        }
        Ok(Decimal::from(self.made)
            .checked_div(total)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::UnexpectedDecimalComputationError,
                ),
            ))?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum ConsensusManagerError {
    InvalidRoundUpdate {
        from: Round,
        to: Round,
    },
    InvalidProposerTimestampUpdate {
        from_millis: i64,
        to_millis: i64,
    },
    InconsistentGapRounds {
        gap_rounds: usize,
        progressed_rounds: u64,
    },
    InvalidValidatorIndex {
        index: ValidatorIndex,
        count: usize,
    },
    AlreadyStarted,
    NotXrd,
    UnexpectedDecimalComputationError,
    EpochMathOverflow,
    InvalidConsensusTime(i64),
    ExceededValidatorCount {
        current: u32,
        max: u32,
    },
}

declare_native_blueprint_state! {
    blueprint_ident: ConsensusManager,
    blueprint_snake_case: consensus_manager,
    features: {
    },
    fields: {
        config: {
            ident: Configuration,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
        state: {
            ident: State,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
        validator_rewards: {
            ident: ValidatorRewards,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
        current_validator_set: {
            ident: CurrentValidatorSet,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
        current_proposal_statistic: {
            ident: CurrentProposalStatistic,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
        proposer_minute_timestamp: {
            ident: ProposerMinuteTimestamp,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
        proposer_milli_timestamp: {
            ident: ProposerMilliTimestamp,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
    },
    collections: {
        registered_validators_by_stake: SortedIndex {
            entry_ident: RegisteredValidatorByStake,
            key_type: {
                kind: Static,
                content_type: ComponentAddress,
            },
            full_key_content: {
                full_content_type: ValidatorByStakeKey,
                sort_prefix_property_name: inverse_stake_sort_prefix,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct ValidatorByStakeKey {
    pub divided_stake: u16,
    pub validator_address: ComponentAddress,
}

impl SortedIndexKeyContentSource<ConsensusManagerRegisteredValidatorByStakeKeyPayload>
    for ValidatorByStakeKey
{
    fn sort_key(&self) -> u16 {
        u16::MAX - self.divided_stake
    }

    fn into_content(
        self,
    ) -> <ConsensusManagerRegisteredValidatorByStakeKeyPayload as SortedIndexKeyPayload>::Content
    {
        self.validator_address
    }
}

impl SortedIndexKeyFullContent<ConsensusManagerRegisteredValidatorByStakeKeyPayload>
    for ValidatorByStakeKey
{
    fn from_sort_key_and_content(sort_key: u16, validator_address: ComponentAddress) -> Self {
        Self {
            divided_stake: u16::MAX - sort_key,
            validator_address,
        }
    }

    fn as_content(&self) -> &ComponentAddress {
        &self.validator_address
    }
}

pub type ConsensusManagerConfigurationV1 = ConsensusManagerConfigSubstate;
pub type ConsensusManagerStateV1 = ConsensusManagerSubstate;
pub type ConsensusManagerValidatorRewardsV1 = ValidatorRewardsSubstate;
pub type ConsensusManagerCurrentValidatorSetV1 = CurrentValidatorSetSubstate;
pub type ConsensusManagerCurrentProposalStatisticV1 = CurrentProposalStatisticSubstate;
pub type ConsensusManagerProposerMinuteTimestampV1 = ProposerMinuteTimestampSubstate;
pub type ConsensusManagerProposerMilliTimestampV1 = ProposerMilliTimestampSubstate;
pub type ConsensusManagerRegisteredValidatorByStakeV1 = Validator;

pub const CONSENSUS_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX: CollectionIndex = 0u8;

pub struct ConsensusManagerBlueprint;

impl ConsensusManagerBlueprint {
    pub fn definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let feature_set = ConsensusManagerFeatureSet::all_features();
        let state = ConsensusManagerStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();
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
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ConsensusManagerGetCurrentEpochInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ConsensusManagerGetCurrentEpochOutput>(),
                ),
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
                        .add_child_type_and_descendents::<ConsensusManagerGetCurrentTimeInputV1>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ConsensusManagerGetCurrentTimeOutput>(),
                ),
                export: CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT.to_string(),
            },
        );
        functions.insert(
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ConsensusManagerCompareCurrentTimeInputV1>(
                        ),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ConsensusManagerCompareCurrentTimeOutput>(
                        ),
                ),
                export: CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT.to_string(),
            },
        );
        functions.insert(
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ConsensusManagerNextRoundInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ConsensusManagerNextRoundOutput>(),
                ),
                export: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
            },
        );
        functions.insert(
            CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ConsensusManagerCreateValidatorInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ConsensusManagerCreateValidatorOutput>(),
                ),
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
            is_transient: false,
            feature_set,
            dependencies: indexset!(
                XRD.into(),
                PACKAGE_OF_DIRECT_CALLER_RESOURCE.into(),
                SYSTEM_EXECUTION_RESOURCE.into(),
                VALIDATOR_OWNER_BADGE.into(),
            ),
            schema: BlueprintSchemaInit {
                generics: vec![],
                schema: consensus_manager_schema,
                state,
                events: event_schema,
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AccessRules(indexmap!(
                    CONSENSUS_MANAGER_CREATE_IDENT.to_string() => rule!(require(system_execution(SystemExecution::Protocol))),
                )),
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template!(
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
    }

    pub(crate) fn create<Y: SystemApi<RuntimeError>>(
        validator_token_address_reservation: GlobalAddressReservation,
        consensus_manager_address_reservation: GlobalAddressReservation,
        genesis_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
        initial_time_milli: i64,
        initial_current_leader: Option<ValidatorIndex>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if initial_config.max_validators > ValidatorIndex::MAX as u32 {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::ExceededValidatorCount {
                        current: initial_config.max_validators,
                        max: ValidatorIndex::MAX as u32,
                    },
                ),
            ));
        }

        {
            // TODO: remove mint and premint all tokens
            let global_id =
                NonFungibleGlobalId::package_of_direct_caller_badge(CONSENSUS_MANAGER_PACKAGE);
            let consensus_manager_address =
                api.get_reservation_address(consensus_manager_address_reservation.0.as_node_id())?;

            ResourceManager::new_non_fungible::<ValidatorOwnerBadgeData, _, _, _>(
                OwnerRole::Fixed(rule!(require(global_caller(consensus_manager_address)))),
                NonFungibleIdType::Bytes,
                true,
                NonFungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(require(global_id));
                        minter_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata_init! {
                    "name" => "Validator Owner Badges".to_owned(), locked;
                    "description" => "Badges created by the Radix system that provide individual control over the validator components created for validator node-runners.".to_owned(), locked;
                    "tags" => vec!["badge".to_owned(), "validator".to_owned()], locked;
                    "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-validator_owner_badge.png".to_owned()), locked;
                },
                Some(validator_token_address_reservation),
                api,
            )?;
        };

        let consensus_manager_id = {
            let config = ConsensusManagerConfigSubstate {
                config: initial_config,
            };
            let consensus_manager = ConsensusManagerSubstate {
                started: false,
                epoch: genesis_epoch,
                actual_epoch_start_milli: initial_time_milli,
                effective_epoch_start_milli: initial_time_milli,
                round: Round::zero(),
                current_leader: initial_current_leader,
            };
            let validator_rewards = ValidatorRewardsSubstate {
                proposer_rewards: index_map_new(),
                rewards_vault: Vault::create(XRD, api)?,
            };
            let current_validator_set = CurrentValidatorSetSubstate {
                validator_set: ActiveValidatorSet {
                    validators_by_stake_desc: index_map_new(),
                },
            };
            let current_proposal_statistic = CurrentProposalStatisticSubstate {
                validator_statistics: Vec::new(),
            };
            let minute_timestamp = ProposerMinuteTimestampSubstate {
                epoch_minute: Self::milli_to_minute(initial_time_milli).ok_or(
                    RuntimeError::ApplicationError(ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::InvalidConsensusTime(initial_time_milli),
                    )),
                )?,
            };
            let milli_timestamp = ProposerMilliTimestampSubstate {
                epoch_milli: initial_time_milli,
            };

            api.new_simple_object(
                CONSENSUS_MANAGER_BLUEPRINT,
                indexmap! {
                    ConsensusManagerField::Configuration.field_index() => FieldValue::immutable(&ConsensusManagerConfigurationFieldPayload::from_content_source(config)),
                    ConsensusManagerField::State.field_index() => FieldValue::new(&ConsensusManagerStateFieldPayload::from_content_source(consensus_manager)),
                    ConsensusManagerField::ValidatorRewards.field_index() => FieldValue::new(&ConsensusManagerValidatorRewardsFieldPayload::from_content_source(validator_rewards)),
                    ConsensusManagerField::CurrentValidatorSet.field_index() => FieldValue::new(&ConsensusManagerCurrentValidatorSetFieldPayload::from_content_source(current_validator_set)),
                    ConsensusManagerField::CurrentProposalStatistic.field_index() => FieldValue::new(&ConsensusManagerCurrentProposalStatisticFieldPayload::from_content_source(current_proposal_statistic)),
                    ConsensusManagerField::ProposerMinuteTimestamp.field_index() => FieldValue::new(&ConsensusManagerProposerMinuteTimestampFieldPayload::from_content_source(minute_timestamp)),
                    ConsensusManagerField::ProposerMilliTimestamp.field_index() => FieldValue::new(&ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(milli_timestamp)),
                },
            )?
        };

        let role_definitions = roles2! {
            VALIDATOR_ROLE => rule!(require(system_execution(SystemExecution::Validator)));
        };

        let roles = indexmap!(ModuleId::Main => role_definitions);
        let role_assignment = RoleAssignment::create(OwnerRole::None, roles, api)?.0;
        let metadata = Metadata::create_with_data(
            metadata_init! {
                "name" => "Consensus Manager".to_owned(), locked;
                "description" => "A component that keeps track of various consensus related concepts such as the epoch, round, current validator set, and so on.".to_owned(), locked;
            },
            api,
        )?;

        api.globalize(
            consensus_manager_id,
            indexmap!(
                AttachedModuleId::RoleAssignment => role_assignment.0,
                AttachedModuleId::Metadata => metadata.0,
            ),
            Some(consensus_manager_address_reservation),
        )?;

        Ok(())
    }

    pub(crate) fn get_current_epoch<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Epoch, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::State.into(),
            LockFlags::read_only(),
        )?;

        let consensus_manager = api
            .field_read_typed::<ConsensusManagerStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        Ok(consensus_manager.epoch)
    }

    pub(crate) fn start<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let config_substate = {
            let config_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                ConsensusManagerField::Configuration.into(),
                LockFlags::read_only(),
            )?;
            let config_substate = api
                .field_read_typed::<ConsensusManagerConfigurationFieldPayload>(config_handle)?
                .fully_update_and_into_latest_version();
            api.field_close(config_handle)?;
            config_substate
        };

        let manager_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::State.into(),
            LockFlags::MUTABLE,
        )?;
        let mut manager_substate = api
            .field_read_typed::<ConsensusManagerStateFieldPayload>(manager_handle)?
            .fully_update_and_into_latest_version();

        if manager_substate.started {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(ConsensusManagerError::AlreadyStarted),
            ));
        }
        let post_genesis_epoch =
            manager_substate
                .epoch
                .next()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::EpochMathOverflow,
                    ),
                ))?;

        Self::epoch_change(post_genesis_epoch, &config_substate.config, api)?;
        manager_substate.started = true;
        manager_substate.epoch = post_genesis_epoch;
        manager_substate.round = Round::zero();

        api.field_write_typed(
            manager_handle,
            &ConsensusManagerStateFieldPayload::from_content_source(manager_substate),
        )?;
        api.field_close(manager_handle)?;

        Ok(())
    }

    pub(crate) fn get_current_time_v1<Y: SystemApi<RuntimeError>>(
        precision: TimePrecisionV1,
        api: &mut Y,
    ) -> Result<Instant, RuntimeError> {
        match precision {
            TimePrecisionV1::Minute => {
                let handle = api.actor_open_field(
                    ACTOR_STATE_SELF,
                    ConsensusManagerField::ProposerMinuteTimestamp.into(),
                    LockFlags::read_only(),
                )?;
                let proposer_minute_timestamp = api
                    .field_read_typed::<ConsensusManagerProposerMinuteTimestampFieldPayload>(
                        handle,
                    )?
                    .fully_update_and_into_latest_version();
                api.field_close(handle)?;

                Ok(Self::epoch_minute_to_instant(
                    proposer_minute_timestamp.epoch_minute,
                ))
            }
        }
    }

    pub(crate) fn get_current_time_v2<Y: SystemApi<RuntimeError>>(
        precision: TimePrecisionV2,
        api: &mut Y,
    ) -> Result<Instant, RuntimeError> {
        match precision {
            TimePrecisionV2::Minute => {
                let handle = api.actor_open_field(
                    ACTOR_STATE_SELF,
                    ConsensusManagerField::ProposerMinuteTimestamp.into(),
                    LockFlags::read_only(),
                )?;
                let proposer_minute_timestamp = api
                    .field_read_typed::<ConsensusManagerProposerMinuteTimestampFieldPayload>(
                        handle,
                    )?
                    .fully_update_and_into_latest_version();
                api.field_close(handle)?;

                Ok(Self::epoch_minute_to_instant(
                    proposer_minute_timestamp.epoch_minute,
                ))
            }
            TimePrecisionV2::Second => {
                let handle = api.actor_open_field(
                    ACTOR_STATE_SELF,
                    ConsensusManagerField::ProposerMilliTimestamp.into(),
                    LockFlags::read_only(),
                )?;
                let proposer_milli_timestamp = api
                    .field_read_typed::<ConsensusManagerProposerMilliTimestampFieldPayload>(handle)?
                    .fully_update_and_into_latest_version();
                api.field_close(handle)?;

                Ok(Self::epoch_milli_to_instant(
                    proposer_milli_timestamp.epoch_milli,
                ))
            }
        }
    }

    pub(crate) fn compare_current_time_v1<Y: SystemApi<RuntimeError>>(
        other_arbitrary_precision_instant: Instant,
        precision: TimePrecisionV1,
        operator: TimeComparisonOperator,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match precision {
            TimePrecisionV1::Minute => {
                let other_epoch_minute = other_arbitrary_precision_instant
                    .seconds_since_unix_epoch
                    .checked_mul(MILLIS_IN_SECOND)
                    .and_then(|result| Self::milli_to_minute(result))
                    .unwrap_or_else(|| {
                        // This is to deal with overflows, i32 MAX and MIN values should work with current time
                        if other_arbitrary_precision_instant
                            .seconds_since_unix_epoch
                            .is_negative()
                        {
                            i32::MIN
                        } else {
                            i32::MAX
                        }
                    });

                let handle = api.actor_open_field(
                    ACTOR_STATE_SELF,
                    ConsensusManagerField::ProposerMinuteTimestamp.into(),
                    LockFlags::read_only(),
                )?;
                let proposer_minute_timestamp = api
                    .field_read_typed::<ConsensusManagerProposerMinuteTimestampFieldPayload>(
                        handle,
                    )?
                    .fully_update_and_into_latest_version();
                api.field_close(handle)?;

                // convert back to Instant only for comparison operation
                let proposer_instant =
                    Self::epoch_minute_to_instant(proposer_minute_timestamp.epoch_minute);
                let other_instant = Self::epoch_minute_to_instant(other_epoch_minute);
                let result = proposer_instant.compare(other_instant, operator);
                Ok(result)
            }
        }
    }

    pub(crate) fn compare_current_time_v2<Y: SystemApi<RuntimeError>>(
        other_arbitrary_precision_instant: Instant,
        precision: TimePrecisionV2,
        operator: TimeComparisonOperator,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match precision {
            TimePrecisionV2::Minute => {
                let other_epoch_minute = other_arbitrary_precision_instant
                    .seconds_since_unix_epoch
                    .checked_mul(MILLIS_IN_SECOND)
                    .and_then(|result| Self::milli_to_minute(result))
                    .unwrap_or_else(|| {
                        // This is to deal with overflows, i32 MAX and MIN values should work with current time
                        if other_arbitrary_precision_instant
                            .seconds_since_unix_epoch
                            .is_negative()
                        {
                            i32::MIN
                        } else {
                            i32::MAX
                        }
                    });

                let handle = api.actor_open_field(
                    ACTOR_STATE_SELF,
                    ConsensusManagerField::ProposerMinuteTimestamp.into(),
                    LockFlags::read_only(),
                )?;
                let proposer_minute_timestamp = api
                    .field_read_typed::<ConsensusManagerProposerMinuteTimestampFieldPayload>(
                        handle,
                    )?
                    .fully_update_and_into_latest_version();
                api.field_close(handle)?;

                // convert back to Instant only for comparison operation
                let proposer_instant =
                    Self::epoch_minute_to_instant(proposer_minute_timestamp.epoch_minute);
                let other_instant = Self::epoch_minute_to_instant(other_epoch_minute);
                let result = proposer_instant.compare(other_instant, operator);
                Ok(result)
            }

            TimePrecisionV2::Second => {
                let other_epoch_second = other_arbitrary_precision_instant.seconds_since_unix_epoch;

                let handle = api.actor_open_field(
                    ACTOR_STATE_SELF,
                    ConsensusManagerField::ProposerMilliTimestamp.into(),
                    LockFlags::read_only(),
                )?;
                let proposer_milli_timestamp = api
                    .field_read_typed::<ConsensusManagerProposerMilliTimestampFieldPayload>(handle)?
                    .fully_update_and_into_latest_version();
                api.field_close(handle)?;

                // convert back to Instant only for comparison operation
                let proposer_instant =
                    Self::epoch_milli_to_instant(proposer_milli_timestamp.epoch_milli);
                let other_instant = Instant::new(other_epoch_second);
                let result = proposer_instant.compare(other_instant, operator);
                Ok(result)
            }
        }
    }

    fn epoch_minute_to_instant(epoch_minute: i32) -> Instant {
        Instant::new(epoch_minute as i64 * SECONDS_IN_MINUTE)
    }

    fn epoch_milli_to_instant(epoch_milli: i64) -> Instant {
        Instant::new(epoch_milli / MILLIS_IN_SECOND)
    }

    fn milli_to_minute(epoch_milli: i64) -> Option<i32> {
        i32::try_from(epoch_milli / MILLIS_IN_MINUTE).ok() // safe until A.D. 5700
    }

    pub(crate) fn next_round<Y: SystemApi<RuntimeError>>(
        round: Round,
        proposer_timestamp_milli: i64,
        proposal_history: LeaderProposalHistory,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::check_non_decreasing_and_update_timestamps(proposer_timestamp_milli, api)?;

        let config_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::Configuration.into(),
            LockFlags::read_only(),
        )?;
        let config_substate = api
            .field_read_typed::<ConsensusManagerConfigurationFieldPayload>(config_handle)?
            .fully_update_and_into_latest_version();
        api.field_close(config_handle)?;

        let manager_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::State.into(),
            LockFlags::MUTABLE,
        )?;
        let mut manager_substate = api
            .field_read_typed::<ConsensusManagerStateFieldPayload>(manager_handle)?
            .fully_update_and_into_latest_version();

        let progressed_rounds = Round::calculate_progress(manager_substate.round, round)
            .ok_or_else(|| {
                RuntimeError::ApplicationError(ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::InvalidRoundUpdate {
                        from: manager_substate.round,
                        to: round,
                    },
                ))
            })?;

        let current_leader = proposal_history.current_leader;
        Self::update_proposal_statistics(progressed_rounds, proposal_history, api)?;

        let config = &config_substate.config;
        let should_epoch_change = config.epoch_change_condition.should_epoch_change(
            manager_substate.effective_epoch_start_milli,
            proposer_timestamp_milli,
            round,
        );
        match should_epoch_change {
            EpochChangeOutcome::NoChange => {
                Runtime::emit_event(api, RoundChangeEvent { round })?;
                manager_substate.round = round;
            }
            EpochChangeOutcome::Change {
                next_epoch_effective_start_millis: next_epoch_effective_start,
            } => {
                let next_epoch =
                    manager_substate
                        .epoch
                        .next()
                        .ok_or(RuntimeError::ApplicationError(
                            ApplicationError::ConsensusManagerError(
                                ConsensusManagerError::EpochMathOverflow,
                            ),
                        ))?;
                Self::epoch_change(next_epoch, config, api)?;
                manager_substate.epoch = next_epoch;
                manager_substate.round = Round::zero();
                manager_substate.actual_epoch_start_milli = proposer_timestamp_milli;
                manager_substate.effective_epoch_start_milli = next_epoch_effective_start;
            }
        }
        manager_substate.current_leader = Some(current_leader);

        api.field_write_typed(
            manager_handle,
            &ConsensusManagerStateFieldPayload::from_content_source(manager_substate),
        )?;
        api.field_close(manager_handle)?;

        Ok(())
    }

    fn get_validator_xrd_cost<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Option<Decimal>, RuntimeError> {
        let manager_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::State.field_index(),
            LockFlags::read_only(),
        )?;
        let manager_substate =
            api.field_read_typed::<ConsensusManagerStateFieldPayload>(manager_handle)?;
        let manager_substate = manager_substate.fully_update_and_into_latest_version();

        let validator_creation_xrd_cost = if manager_substate.started {
            let config_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                ConsensusManagerField::Configuration.into(),
                LockFlags::read_only(),
            )?;
            let manager_config: ConsensusManagerConfigurationFieldPayload =
                api.field_read_typed(config_handle)?;
            api.field_close(config_handle)?;

            let validator_creation_xrd_cost = manager_config
                .fully_update_and_into_latest_version()
                .config
                .validator_creation_usd_cost
                .checked_mul(api.usd_price()?)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::UnexpectedDecimalComputationError,
                    ),
                ))?;
            Some(validator_creation_xrd_cost)
        } else {
            None
        };

        api.field_close(manager_handle)?;

        Ok(validator_creation_xrd_cost)
    }

    pub(crate) fn create_validator<Y: SystemApi<RuntimeError>>(
        key: Secp256k1PublicKey,
        fee_factor: Decimal,
        xrd_payment: Bucket,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket, Bucket), RuntimeError> {
        if !xrd_payment.resource_address(api)?.eq(&XRD) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(ConsensusManagerError::NotXrd),
            ));
        }

        let validator_xrd_cost = Self::get_validator_xrd_cost(api)?;
        if let Some(xrd_cost) = validator_xrd_cost {
            let xrd_paid = xrd_payment.take(xrd_cost, api)?;
            xrd_paid.burn(api)?;
        }

        let (validator_address, owner_token_bucket) =
            ValidatorCreator::create(key, false, fee_factor, api)?;

        Ok((validator_address, owner_token_bucket, xrd_payment))
    }

    fn check_non_decreasing_and_update_timestamps<Y: SystemApi<RuntimeError>>(
        current_time_ms: i64,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::ProposerMilliTimestamp.into(),
            LockFlags::MUTABLE,
        )?;
        let exact_time_substate: ConsensusManagerProposerMilliTimestampFieldPayload =
            api.field_read_typed(handle)?;
        let mut exact_time_substate = exact_time_substate.fully_update_and_into_latest_version();
        let previous_timestamp = exact_time_substate.epoch_milli;
        if current_time_ms < previous_timestamp {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::InvalidProposerTimestampUpdate {
                        from_millis: previous_timestamp,
                        to_millis: current_time_ms,
                    },
                ),
            ));
        } else if current_time_ms > previous_timestamp {
            exact_time_substate.epoch_milli = current_time_ms;
            api.field_write_typed(
                handle,
                &ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(
                    exact_time_substate,
                ),
            )?;
        }
        api.field_close(handle)?;

        let new_rounded_value = Self::milli_to_minute(current_time_ms).ok_or(
            RuntimeError::ApplicationError(ApplicationError::ConsensusManagerError(
                ConsensusManagerError::InvalidConsensusTime(current_time_ms),
            )),
        )?;
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::ProposerMinuteTimestamp.into(),
            LockFlags::MUTABLE,
        )?;
        let rounded_timestamp_substate: ConsensusManagerProposerMinuteTimestampFieldPayload =
            api.field_read_typed(handle)?;
        let mut rounded_timestamp_substate =
            rounded_timestamp_substate.fully_update_and_into_latest_version();
        let previous_rounded_value = rounded_timestamp_substate.epoch_minute;
        if new_rounded_value > previous_rounded_value {
            rounded_timestamp_substate.epoch_minute = new_rounded_value;
            api.field_write_typed(
                handle,
                &ConsensusManagerProposerMinuteTimestampFieldPayload::from_content_source(
                    rounded_timestamp_substate,
                ),
            )?;
        }
        api.field_close(handle)?;

        Ok(())
    }

    fn update_proposal_statistics<Y: SystemApi<RuntimeError>>(
        progressed_rounds: u64,
        proposal_history: LeaderProposalHistory,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if proposal_history.gap_round_leaders.len() as u64 != progressed_rounds - 1 {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::InconsistentGapRounds {
                        gap_rounds: proposal_history.gap_round_leaders.len(),
                        progressed_rounds,
                    },
                ),
            ));
        }

        let statistic_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::CurrentProposalStatistic.into(),
            LockFlags::MUTABLE,
        )?;
        let statistic: ConsensusManagerCurrentProposalStatisticFieldPayload =
            api.field_read_typed(statistic_handle)?;
        let mut statistic = statistic.fully_update_and_into_latest_version();
        for gap_round_leader in proposal_history.gap_round_leaders {
            let gap_round_statistic = statistic.get_mut_proposal_statistic(gap_round_leader)?;
            gap_round_statistic.missed += 1;
        }
        let current_round_statistic =
            statistic.get_mut_proposal_statistic(proposal_history.current_leader)?;
        if proposal_history.is_fallback {
            current_round_statistic.missed += 1;
        } else {
            current_round_statistic.made += 1;
        }
        api.field_write_typed(
            statistic_handle,
            &ConsensusManagerCurrentProposalStatisticFieldPayload::from_content_source(statistic),
        )?;
        api.field_close(statistic_handle)?;

        Ok(())
    }

    fn epoch_change<Y: SystemApi<RuntimeError>>(
        next_epoch: Epoch,
        config: &ConsensusManagerConfig,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // Read previous validator set
        let validator_set_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::CurrentValidatorSet.into(),
            LockFlags::MUTABLE,
        )?;
        let validator_set_substate: ConsensusManagerCurrentValidatorSetFieldPayload =
            api.field_read_typed(validator_set_handle)?;
        let mut validator_set_substate =
            validator_set_substate.fully_update_and_into_latest_version();
        let previous_validator_set = validator_set_substate.validator_set;

        // Read previous validator statistics
        let statistic_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::CurrentProposalStatistic.into(),
            LockFlags::MUTABLE,
        )?;
        let statistic_substate: ConsensusManagerCurrentProposalStatisticFieldPayload =
            api.field_read_typed(statistic_handle)?;
        let mut statistic_substate = statistic_substate.fully_update_and_into_latest_version();
        let previous_statistics = statistic_substate.validator_statistics;

        // Read & write validator rewards
        let rewards_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ConsensusManagerField::ValidatorRewards.into(),
            LockFlags::MUTABLE,
        )?;
        let mut rewards_substate = api
            .field_read_typed::<ConsensusManagerValidatorRewardsFieldPayload>(rewards_handle)?
            .fully_update_and_into_latest_version();

        // Apply emissions
        Self::apply_validator_emissions_and_rewards(
            previous_validator_set,
            previous_statistics,
            config,
            &mut rewards_substate,
            next_epoch.previous().ok_or(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(ConsensusManagerError::EpochMathOverflow),
            ))?,
            api,
        )?;

        // Select next validator set
        // NOTE - because the stake index is by u16 buckets, it's possible that there are multiple validators at the cut off point
        // that fall into the same bucket.
        // To reduce the risk of that causing issues, we take a decent chunk more than we need from the index.
        // It's still possible that the bucket is _very_ large and we miss some validators in the bucket, and fail to read validators
        // with a higher stake, but lower DbSortKey.
        // The risk is very low though in practice, and only affects validators near the bottom of the list who would likely get very
        // few proposals, so we feel it's an okay trade-off.
        let num_validators_to_read_from_store =
            config.max_validators + (config.max_validators / 10) + 10;

        let mut top_registered_validators: Vec<(
            ComponentAddress,
            ConsensusManagerRegisteredValidatorByStakeEntryPayload,
        )> = api.actor_sorted_index_scan_typed(
            ACTOR_STATE_SELF,
            ConsensusManagerCollection::RegisteredValidatorByStakeSortedIndex.collection_index(),
            num_validators_to_read_from_store,
        )?;

        // The index scan should already pull the validators out in stake DESC, but if multiple validators are on the same u16 stake,
        // then let's be even more accurate here. This sort is stable, so if two validators tie, then the resultant order will be
        // decided on sort key DESC.
        top_registered_validators.sort_by(|(_, validator_1), (_, validator_2)| {
            let validator1 = validator_1.as_unique_version();
            let validator2 = validator_2.as_unique_version();
            validator1.stake.cmp(&validator2.stake).reverse()
        });

        let next_active_validator_set = ActiveValidatorSet {
            validators_by_stake_desc: top_registered_validators
                .into_iter()
                .take(config.max_validators as usize)
                .map(|(component_address, validator)| {
                    (
                        component_address,
                        validator.fully_update_and_into_latest_version(),
                    )
                })
                .collect(),
        };

        let mut next_validator_set_total_stake = Decimal::zero();
        let mut significant_protocol_update_readiness: IndexMap<String, Decimal> = index_map_new();
        for (validator_address, validator) in
            next_active_validator_set.validators_by_stake_desc.iter()
        {
            next_validator_set_total_stake = next_validator_set_total_stake
                .checked_add(validator.stake)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::UnexpectedDecimalComputationError,
                    ),
                ))?;
            let rtn = api.call_method(
                validator_address.as_node_id(),
                VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT,
                scrypto_encode(&ValidatorGetProtocolUpdateReadinessInput {}).unwrap(),
            )?;
            if let Some(protocol_update_readiness) = scrypto_decode::<Option<String>>(&rtn).unwrap()
            {
                let entry = significant_protocol_update_readiness
                    .entry(protocol_update_readiness)
                    .or_insert(Decimal::zero());
                *entry =
                    entry
                        .checked_add(validator.stake)
                        .ok_or(RuntimeError::ApplicationError(
                            ApplicationError::ConsensusManagerError(
                                ConsensusManagerError::UnexpectedDecimalComputationError,
                            ),
                        ))?;
            }
        }

        // Only store protocol updates that have been signalled by at
        // least 10% of the new epoch's validator set total stake.
        let significant_protocol_update_readiness_stake_threshold = next_validator_set_total_stake
            .checked_mul(dec!("0.1"))
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::UnexpectedDecimalComputationError,
                ),
            ))?;
        significant_protocol_update_readiness.retain(|_, stake_signalled| {
            *stake_signalled >= significant_protocol_update_readiness_stake_threshold
        });

        // Emit epoch change event
        Runtime::emit_event(
            api,
            EpochChangeEvent {
                epoch: next_epoch,
                validator_set: next_active_validator_set.clone(),
                significant_protocol_update_readiness,
            },
        )?;

        // Write updated validator rewards
        api.field_write_typed(
            rewards_handle,
            &ConsensusManagerValidatorRewardsFieldPayload::from_content_source(rewards_substate),
        )?;
        api.field_close(rewards_handle)?;

        // Write zeroed statistics of next validators
        statistic_substate.validator_statistics = (0..next_active_validator_set.validator_count())
            .map(|_index| ProposalStatistic::default())
            .collect();
        api.field_write_typed(
            statistic_handle,
            &ConsensusManagerCurrentProposalStatisticFieldPayload::from_content_source(
                statistic_substate,
            ),
        )?;
        api.field_close(statistic_handle)?;

        // Write next validator set
        validator_set_substate.validator_set = next_active_validator_set;
        api.field_write_typed(
            validator_set_handle,
            &ConsensusManagerCurrentValidatorSetFieldPayload::from_content_source(
                validator_set_substate,
            ),
        )?;
        api.field_close(validator_set_handle)?;

        Ok(())
    }

    /// Emits a configured XRD amount ([`ConsensusManagerConfigSubstate.total_emission_xrd_per_epoch`])
    /// and distributes it across the given validator set, according to their stake.
    fn apply_validator_emissions_and_rewards<Y: SystemApi<RuntimeError>>(
        validator_set: ActiveValidatorSet,
        validator_statistics: Vec<ProposalStatistic>,
        config: &ConsensusManagerConfig,
        validator_rewards: &mut ValidatorRewardsSubstate,
        epoch: Epoch, // the concluded epoch, for event creation
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let mut stake_sum_xrd = Decimal::ZERO;

        let mut validator_infos: IndexMap<ValidatorIndex, ValidatorInfo> = index_map_new();
        for (index, (address, validator)) in validator_set
            .validators_by_stake_desc
            .into_iter()
            .enumerate()
        {
            if let Some(info) = ValidatorInfo::create_if_applicable(
                address,
                validator.stake,
                validator_statistics[index].clone(),
                config.min_validator_reliability,
            )? {
                stake_sum_xrd = stake_sum_xrd.checked_add(info.stake_xrd).ok_or(
                    RuntimeError::ApplicationError(ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::UnexpectedDecimalComputationError,
                    )),
                )?;

                validator_infos.insert(
                    TryInto::<ValidatorIndex>::try_into(index)
                        // Should never happen. We made sure no more than u8::MAX validators are stored
                        .expect("Validator index exceeds the range of u8"),
                    info,
                );
            } else {
                // Excluded due to slashing ?
            }
        }
        if validator_infos.is_empty() {
            return Ok(());
        }

        //======================
        // Distribute emissions
        //======================

        // calculate "how much XRD is emitted by 1 XRD staked", and later apply it evenly among validators
        // (the gains are slightly rounded down, but more fairly distributed - not affected by different rounding errors for different validators)
        let emission_per_staked_xrd = config
            .total_emission_xrd_per_epoch
            .checked_div(stake_sum_xrd)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::UnexpectedDecimalComputationError,
                ),
            ))?;
        let effective_total_emission_xrd = {
            let mut sum = Decimal::ZERO;

            for v in validator_infos.values() {
                let emission = v
                    .effective_stake_xrd
                    .checked_mul(emission_per_staked_xrd)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::ConsensusManagerError(
                            ConsensusManagerError::UnexpectedDecimalComputationError,
                        ),
                    ))?;
                sum = sum
                    .checked_add(emission)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::ConsensusManagerError(
                            ConsensusManagerError::UnexpectedDecimalComputationError,
                        ),
                    ))?;
            }
            sum
        };

        let total_emission_xrd_bucket =
            ResourceManager(XRD).mint_fungible(effective_total_emission_xrd, api)?;

        for validator_info in validator_infos.values() {
            let emission_xrd_bucket = total_emission_xrd_bucket.take(
                validator_info
                    .effective_stake_xrd
                    .checked_mul(emission_per_staked_xrd)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::ConsensusManagerError(
                            ConsensusManagerError::UnexpectedDecimalComputationError,
                        ),
                    ))?,
                api,
            )?;
            api.call_method(
                validator_info.address.as_node_id(),
                VALIDATOR_APPLY_EMISSION_IDENT,
                scrypto_encode(&ValidatorApplyEmissionInput {
                    xrd_bucket: emission_xrd_bucket.into(),
                    epoch,
                    proposals_made: validator_info.proposal_statistic.made,
                    proposals_missed: validator_info.proposal_statistic.missed,
                })
                .unwrap(),
            )?;
        }
        total_emission_xrd_bucket.drop_empty(api)?;

        //===========================
        // Distribute rewards (fees)
        //===========================
        let mut total_effective_stake = Decimal::ZERO;
        let mut total_claimable_proposer_rewards = Decimal::ZERO;

        // Note that `validator_infos` are for applicable validators (i.e. stake > 0) only
        // Being an applicable validator doesn't necessarily mean the effective stake is positive, due to reliability rescaling.
        for (index, validator_info) in &validator_infos {
            total_effective_stake = total_effective_stake
                .checked_add(validator_info.effective_stake_xrd)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::UnexpectedDecimalComputationError,
                    ),
                ))?;
            total_claimable_proposer_rewards = total_claimable_proposer_rewards
                .checked_add(
                    validator_rewards
                        .proposer_rewards
                        .get(index)
                        .cloned()
                        .unwrap_or_default(),
                )
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::UnexpectedDecimalComputationError,
                    ),
                ))?;
        }

        let total_claimable_validator_set_rewards = validator_rewards
            .rewards_vault
            .amount(api)?
            .checked_sub(total_claimable_proposer_rewards)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::UnexpectedDecimalComputationError,
                ),
            ))?;
        let reward_per_effective_stake = if total_effective_stake.is_zero() {
            // This is another extreme use case.
            // Can the network even progress if total effective stake is zero?
            Decimal::ZERO
        } else {
            total_claimable_validator_set_rewards
                .checked_div(total_effective_stake)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::UnexpectedDecimalComputationError,
                    ),
                ))?
        };

        for (index, validator_info) in validator_infos {
            let as_proposer = validator_rewards
                .proposer_rewards
                .swap_remove(&index)
                .unwrap_or_default();
            let as_member_of_validator_set = validator_info
                .effective_stake_xrd
                .checked_mul(reward_per_effective_stake)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::UnexpectedDecimalComputationError,
                    ),
                ))?;
            let total_rewards = as_proposer.checked_add(as_member_of_validator_set).ok_or(
                RuntimeError::ApplicationError(ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::UnexpectedDecimalComputationError,
                )),
            )?;
            if total_rewards.is_zero() {
                continue;
            }

            // Note that dusted xrd (due to rounding) are kept in the vault and will
            // become retrievable next time.
            let xrd_bucket = validator_rewards.rewards_vault.take(total_rewards, api)?;

            api.call_method(
                validator_info.address.as_node_id(),
                VALIDATOR_APPLY_REWARD_IDENT,
                scrypto_encode(&ValidatorApplyRewardInput { xrd_bucket, epoch }).unwrap(),
            )?;
        }

        // For any reason, if a validator isn't included in the `validator_infos` but has accumulated
        // proposer rewards, we reset the counter as the rewards has been distributed to other validators.
        validator_rewards.proposer_rewards.clear();

        Ok(())
    }
}

#[derive(Debug)]
struct ValidatorInfo {
    pub address: ComponentAddress,
    pub stake_xrd: Decimal,
    pub effective_stake_xrd: Decimal,
    pub proposal_statistic: ProposalStatistic, // needed only for passing the information to event
}

impl ValidatorInfo {
    fn create_if_applicable(
        address: ComponentAddress,
        stake_xrd: Decimal,
        proposal_statistic: ProposalStatistic,
        min_required_reliability: Decimal,
    ) -> Result<Option<Self>, RuntimeError> {
        if stake_xrd.is_positive() {
            let reliability_factor = Self::to_reliability_factor(
                proposal_statistic.success_ratio()?,
                min_required_reliability,
            )?;
            let effective_stake_xrd =
                stake_xrd
                    .checked_mul(reliability_factor)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::ConsensusManagerError(
                            ConsensusManagerError::UnexpectedDecimalComputationError,
                        ),
                    ))?;
            Ok(Some(Self {
                address,
                stake_xrd,
                proposal_statistic,
                effective_stake_xrd,
            }))
        } else {
            Ok(None)
        }
    }

    /// Converts the absolute reliability measure (e.g. "0.97 uptime") into a reliability factor
    /// which directly drives the fraction of received emission (e.g. "0.25 of base emission"), by
    /// rescaling it into the allowed reliability range (e.g. "required >0.96 uptime").
    fn to_reliability_factor(
        reliability: Decimal,
        min_required_reliability: Decimal,
    ) -> Result<Decimal, RuntimeError> {
        let reliability_reserve = reliability.checked_sub(min_required_reliability).ok_or(
            RuntimeError::ApplicationError(ApplicationError::ConsensusManagerError(
                ConsensusManagerError::UnexpectedDecimalComputationError,
            )),
        )?;
        if reliability_reserve.is_negative() {
            return Ok(Decimal::zero());
        }
        let max_allowed_unreliability =
            Decimal::one().checked_sub(min_required_reliability).ok_or(
                RuntimeError::ApplicationError(ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::UnexpectedDecimalComputationError,
                )),
            )?;
        if max_allowed_unreliability.is_zero() {
            // special-casing the dirac delta behavior
            if reliability == Decimal::one() {
                return Ok(Decimal::one());
            } else {
                return Ok(Decimal::zero());
            }
        }
        Ok(reliability_reserve
            .checked_div(max_allowed_unreliability)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::UnexpectedDecimalComputationError,
                ),
            ))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_reliability_factor() {
        let min_required_reliability = dec!("0.8");
        assert_eq!(
            ValidatorInfo::to_reliability_factor(dec!("0"), min_required_reliability),
            Ok(dec!("0"))
        );
        assert_eq!(
            ValidatorInfo::to_reliability_factor(dec!("0.4"), min_required_reliability),
            Ok(dec!("0"))
        );

        // Is the following rescaling desired?
        assert_eq!(
            ValidatorInfo::to_reliability_factor(dec!("0.8"), min_required_reliability),
            Ok(dec!("0"))
        );
        assert_eq!(
            ValidatorInfo::to_reliability_factor(dec!("0.9"), min_required_reliability),
            Ok(dec!("0.5"))
        );
        assert_eq!(
            ValidatorInfo::to_reliability_factor(dec!("0.95"), min_required_reliability),
            Ok(dec!("0.75"))
        );
        assert_eq!(
            ValidatorInfo::to_reliability_factor(dec!("1"), min_required_reliability),
            Ok(dec!("1"))
        );
    }
}
