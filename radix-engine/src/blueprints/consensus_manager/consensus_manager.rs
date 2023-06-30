use super::{EpochChangeEvent, RoundChangeEvent, ValidatorCreator, ValidatorOwnerBadgeData};
use crate::blueprints::consensus_manager::VALIDATOR_ROLE;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::NativeVault;
use native_sdk::resource::{NativeBucket, ResourceManager};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::node_modules::metadata::Url;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientApi, CollectionIndex, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::{metadata_init, rule};

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

impl Clone for ValidatorRewardsSubstate {
    fn clone(&self) -> Self {
        Self {
            proposer_rewards: self.proposer_rewards.clone(),
            rewards_vault: Vault(self.rewards_vault.0.clone()),
        }
    }
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
    pub fn total_active_stake_xrd(&self) -> Decimal {
        self.validators_by_stake_desc
            .iter()
            .map(|(_, validator)| validator.stake)
            .sum()
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
    pub fn success_ratio(&self) -> Decimal {
        let total = self.made + self.missed;
        if total == 0 {
            return Decimal::one();
        }
        Decimal::from(self.made) / Decimal::from(total)
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
    InvalidXrdPayment {
        expected: Decimal,
        actual: Decimal,
    },
}

pub const CONSENSUS_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX: CollectionIndex = 0u8;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EpochRegisteredValidatorByStakeEntry {
    pub component_address: ComponentAddress,
    pub validator: Validator,
}

pub struct ConsensusManagerBlueprint;

impl ConsensusManagerBlueprint {
    pub(crate) fn create<Y>(
        validator_token_address_reservation: GlobalAddressReservation,
        consensus_manager_address_reservation: GlobalAddressReservation,
        genesis_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
        initial_time_milli: i64,
        initial_current_leader: Option<ValidatorIndex>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        {
            let mut access_rules = BTreeMap::new();

            // TODO: remove mint and premint all tokens
            {
                let global_id =
                    NonFungibleGlobalId::package_of_direct_caller_badge(CONSENSUS_MANAGER_PACKAGE);
                access_rules.insert(Mint, (rule!(require(global_id)), rule!(deny_all)));
            }

            access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));

            let consensus_manager_address =
                api.get_reservation_address(consensus_manager_address_reservation.0.as_node_id())?;

            ResourceManager::new_non_fungible::<ValidatorOwnerBadgeData, Y, RuntimeError, _>(
                OwnerRole::Fixed(rule!(require(global_caller(consensus_manager_address)))),
                NonFungibleIdType::Bytes,
                true,
                access_rules,
                metadata_init! {
                    "name" => "Validator Owner Badges".to_owned(), locked;
                    "description" => "Badges created by the Radix system that provide individual control over the validator components created for validator node-runners.".to_owned(), locked;
                    "tags" => vec!["badge".to_owned(), "validator".to_owned()], locked;
                    "icon_url" => Url("https://assets.radixdlt.com/icons/icon-validator_owner_badge.png".to_owned()), locked;
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
                rewards_vault: Vault::create(RADIX_TOKEN, api)?,
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
                epoch_minute: Self::milli_to_minute(initial_time_milli),
            };
            let milli_timestamp = ProposerMilliTimestampSubstate {
                epoch_milli: initial_time_milli,
            };

            api.new_simple_object(
                CONSENSUS_MANAGER_BLUEPRINT,
                vec![
                    scrypto_encode(&config).unwrap(),
                    scrypto_encode(&consensus_manager).unwrap(),
                    scrypto_encode(&validator_rewards).unwrap(),
                    scrypto_encode(&current_validator_set).unwrap(),
                    scrypto_encode(&current_proposal_statistic).unwrap(),
                    scrypto_encode(&minute_timestamp).unwrap(),
                    scrypto_encode(&milli_timestamp).unwrap(),
                ],
            )?
        };

        let role_definitions = roles2! {
            VALIDATOR_ROLE => rule!(require(AuthAddresses::validator_role()));
        };

        let roles = btreemap!(ObjectModuleId::Main => role_definitions);
        let access_rules = AccessRules::create(OwnerRole::None, roles, api)?.0;
        let metadata = Metadata::create_with_data(
            metadata_init! {
                "name" => "Consensus Manager".to_owned(), locked;
                "description" => "A component that keeps track of various consensus related concepts such as the epoch, round, current validator set, and so on.".to_owned(), locked;
            },
            api,
        )?;
        let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;

        api.globalize(
            btreemap!(
                ObjectModuleId::Main => consensus_manager_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            Some(consensus_manager_address_reservation),
        )?;

        Ok(())
    }

    pub(crate) fn get_current_epoch<Y>(api: &mut Y) -> Result<Epoch, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::ConsensusManager.into(),
            LockFlags::read_only(),
        )?;

        let consensus_manager: ConsensusManagerSubstate = api.field_lock_read_typed(handle)?;

        Ok(consensus_manager.epoch)
    }

    pub(crate) fn start<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let config_substate = {
            let config_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                ConsensusManagerField::Config.into(),
                LockFlags::read_only(),
            )?;
            let config_substate: ConsensusManagerConfigSubstate =
                api.field_lock_read_typed(config_handle)?;
            api.field_lock_release(config_handle)?;
            config_substate
        };

        let manager_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::ConsensusManager.into(),
            LockFlags::MUTABLE,
        )?;
        let mut manager_substate: ConsensusManagerSubstate =
            api.field_lock_read_typed(manager_handle)?;

        if manager_substate.started {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(ConsensusManagerError::AlreadyStarted),
            ));
        }
        let post_genesis_epoch = manager_substate.epoch.next();

        Self::epoch_change(post_genesis_epoch, &config_substate.config, api)?;
        manager_substate.started = true;
        manager_substate.epoch = post_genesis_epoch;
        manager_substate.round = Round::zero();

        api.field_lock_write_typed(manager_handle, manager_substate)?;
        api.field_lock_release(manager_handle)?;

        Ok(())
    }

    pub(crate) fn get_current_time<Y>(
        precision: TimePrecision,
        api: &mut Y,
    ) -> Result<Instant, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match precision {
            TimePrecision::Minute => {
                let handle = api.actor_open_field(
                    OBJECT_HANDLE_SELF,
                    ConsensusManagerField::CurrentTimeRoundedToMinutes.into(),
                    LockFlags::read_only(),
                )?;
                let proposer_minute_timestamp: ProposerMinuteTimestampSubstate =
                    api.field_lock_read_typed(handle)?;
                api.field_lock_release(handle)?;

                Ok(Self::epoch_minute_to_instant(
                    proposer_minute_timestamp.epoch_minute,
                ))
            }
        }
    }

    pub(crate) fn compare_current_time<Y>(
        other_arbitrary_precision_instant: Instant,
        precision: TimePrecision,
        operator: TimeComparisonOperator,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match precision {
            TimePrecision::Minute => {
                let other_epoch_minute = Self::milli_to_minute(
                    other_arbitrary_precision_instant.seconds_since_unix_epoch * MILLIS_IN_SECOND,
                );

                let handle = api.actor_open_field(
                    OBJECT_HANDLE_SELF,
                    ConsensusManagerField::CurrentTimeRoundedToMinutes.into(),
                    LockFlags::read_only(),
                )?;
                let proposer_minute_timestamp: ProposerMinuteTimestampSubstate =
                    api.field_lock_read_typed(handle)?;
                api.field_lock_release(handle)?;

                // convert back to Instant only for comparison operation
                let proposer_instant =
                    Self::epoch_minute_to_instant(proposer_minute_timestamp.epoch_minute);
                let other_instant = Self::epoch_minute_to_instant(other_epoch_minute);
                let result = proposer_instant.compare(other_instant, operator);
                Ok(result)
            }
        }
    }

    fn epoch_minute_to_instant(epoch_minute: i32) -> Instant {
        Instant::new(epoch_minute as i64 * SECONDS_IN_MINUTE)
    }

    fn milli_to_minute(epoch_milli: i64) -> i32 {
        i32::try_from(epoch_milli / MILLIS_IN_MINUTE).unwrap() // safe until A.D. 5700
    }

    pub(crate) fn next_round<Y>(
        round: Round,
        proposer_timestamp_milli: i64,
        proposal_history: LeaderProposalHistory,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::check_non_decreasing_and_update_timestamps(proposer_timestamp_milli, api)?;

        let config_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::Config.into(),
            LockFlags::read_only(),
        )?;
        let config_substate: ConsensusManagerConfigSubstate =
            api.field_lock_read_typed(config_handle)?;
        api.field_lock_release(config_handle)?;

        let manager_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::ConsensusManager.into(),
            LockFlags::MUTABLE,
        )?;
        let mut manager_substate: ConsensusManagerSubstate =
            api.field_lock_read_typed(manager_handle)?;

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
                let next_epoch = manager_substate.epoch.next();
                Self::epoch_change(next_epoch, config, api)?;
                manager_substate.epoch = next_epoch;
                manager_substate.round = Round::zero();
                manager_substate.actual_epoch_start_milli = proposer_timestamp_milli;
                manager_substate.effective_epoch_start_milli = next_epoch_effective_start;
            }
        }
        manager_substate.current_leader = Some(current_leader);

        api.field_lock_write_typed(manager_handle, &manager_substate)?;
        api.field_lock_release(manager_handle)?;

        Ok(())
    }

    fn get_validator_xrd_cost<Y>(api: &mut Y) -> Result<Option<Decimal>, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let manager_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::ConsensusManager.into(),
            LockFlags::read_only(),
        )?;
        let manager_substate: ConsensusManagerSubstate =
            api.field_lock_read_typed(manager_handle)?;

        let validator_creation_xrd_cost = if manager_substate.started {
            let config_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                ConsensusManagerField::Config.into(),
                LockFlags::read_only(),
            )?;
            let manager_config: ConsensusManagerConfigSubstate =
                api.field_lock_read_typed(config_handle)?;
            api.field_lock_release(config_handle)?;
            Some(manager_config.config.validator_creation_xrd_cost)
        } else {
            None
        };

        api.field_lock_release(manager_handle)?;

        Ok(validator_creation_xrd_cost)
    }

    pub(crate) fn create_validator<Y>(
        key: Secp256k1PublicKey,
        fee_factor: Decimal,
        xrd_payment: Bucket,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        if !xrd_payment.resource_address(api)?.eq(&XRD) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(ConsensusManagerError::NotXrd),
            ));
        }

        let validator_xrd_cost = Self::get_validator_xrd_cost(api)?;
        if let Some(xrd_cost) = validator_xrd_cost {
            let payment_amount = xrd_payment.amount(api)?;
            if !payment_amount.eq(&xrd_cost) {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ConsensusManagerError(
                        ConsensusManagerError::InvalidXrdPayment {
                            actual: payment_amount,
                            expected: xrd_cost,
                        },
                    ),
                ));
            }

            xrd_payment.burn(api)?;
        } else {
            xrd_payment.drop_empty(api)?;
        }

        let (validator_address, owner_token_bucket) =
            ValidatorCreator::create(key, false, fee_factor, api)?;

        Ok((validator_address, owner_token_bucket))
    }

    fn check_non_decreasing_and_update_timestamps<Y>(
        current_time_ms: i64,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::CurrentTime.into(),
            LockFlags::MUTABLE,
        )?;
        let mut exact_time_substate: ProposerMilliTimestampSubstate =
            api.field_lock_read_typed(handle)?;
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
            api.field_lock_write_typed(handle, &exact_time_substate)?;
        }
        api.field_lock_release(handle)?;

        let new_rounded_value = Self::milli_to_minute(current_time_ms);
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::CurrentTimeRoundedToMinutes.into(),
            LockFlags::MUTABLE,
        )?;
        let mut rounded_timestamp_substate: ProposerMinuteTimestampSubstate =
            api.field_lock_read_typed(handle)?;
        let previous_rounded_value = rounded_timestamp_substate.epoch_minute;
        if new_rounded_value > previous_rounded_value {
            rounded_timestamp_substate.epoch_minute = new_rounded_value;
            api.field_lock_write_typed(handle, &rounded_timestamp_substate)?;
        }
        api.field_lock_release(handle)?;

        Ok(())
    }

    fn update_proposal_statistics<Y>(
        progressed_rounds: u64,
        proposal_history: LeaderProposalHistory,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
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
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::CurrentProposalStatistic.into(),
            LockFlags::MUTABLE,
        )?;
        let mut statistic: CurrentProposalStatisticSubstate =
            api.field_lock_read_typed(statistic_handle)?;
        for gap_round_leader in proposal_history.gap_round_leaders {
            let mut gap_round_statistic = statistic.get_mut_proposal_statistic(gap_round_leader)?;
            gap_round_statistic.missed += 1;
        }
        let mut current_round_statistic =
            statistic.get_mut_proposal_statistic(proposal_history.current_leader)?;
        if proposal_history.is_fallback {
            current_round_statistic.missed += 1;
        } else {
            current_round_statistic.made += 1;
        }
        api.field_lock_write_typed(statistic_handle, statistic)?;
        api.field_lock_release(statistic_handle)?;

        Ok(())
    }

    fn epoch_change<Y>(
        next_epoch: Epoch,
        config: &ConsensusManagerConfig,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Read previous validator set
        let validator_set_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::CurrentValidatorSet.into(),
            LockFlags::MUTABLE,
        )?;
        let mut validator_set_substate: CurrentValidatorSetSubstate =
            api.field_lock_read_typed(validator_set_handle)?;
        let previous_validator_set = validator_set_substate.validator_set;

        // Read previous validator statistics
        let statistic_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::CurrentProposalStatistic.into(),
            LockFlags::MUTABLE,
        )?;
        let mut statistic_substate: CurrentProposalStatisticSubstate =
            api.field_lock_read_typed(statistic_handle)?;
        let previous_statistics = statistic_substate.validator_statistics;

        // Read & write validator rewards
        let rewards_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::ValidatorRewards.into(),
            LockFlags::MUTABLE,
        )?;
        let mut rewards_substate: ValidatorRewardsSubstate =
            api.field_lock_read_typed(rewards_handle)?;

        // Apply emissions
        Self::apply_validator_emissions_and_rewards(
            previous_validator_set,
            previous_statistics,
            config,
            &mut rewards_substate,
            next_epoch.previous(),
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

        let mut top_registered_validators: Vec<EpochRegisteredValidatorByStakeEntry> = api
            .actor_sorted_index_scan_typed(
                OBJECT_HANDLE_SELF,
                CONSENSUS_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX,
                num_validators_to_read_from_store,
            )?;

        // The index scan should already pull the validators out in stake DESC, but if multiple validators are on the same u16 stake,
        // then let's be even more accurate here. This sort is stable, so if two validators tie, then the resultant order will be
        // decided on sort key DESC.
        top_registered_validators.sort_by(|validator_1, validator_2| {
            validator_1
                .validator
                .stake
                .cmp(&validator_2.validator.stake)
                .reverse()
        });

        let next_active_validator_set = ActiveValidatorSet {
            validators_by_stake_desc: top_registered_validators
                .into_iter()
                .take(config.max_validators as usize)
                .map(|entry| (entry.component_address, entry.validator))
                .collect(),
        };

        // Emit epoch change event
        Runtime::emit_event(
            api,
            EpochChangeEvent {
                epoch: next_epoch,
                validator_set: next_active_validator_set.clone(),
            },
        )?;

        // Write updated validator rewards
        api.field_lock_write_typed(rewards_handle, rewards_substate)?;
        api.field_lock_release(rewards_handle)?;

        // Write zeroed statistics of next validators
        statistic_substate.validator_statistics = (0..next_active_validator_set.validator_count())
            .map(|_index| ProposalStatistic::default())
            .collect();
        api.field_lock_write_typed(statistic_handle, statistic_substate)?;
        api.field_lock_release(statistic_handle)?;

        // Write next validator set
        validator_set_substate.validator_set = next_active_validator_set;
        api.field_lock_write_typed(validator_set_handle, validator_set_substate)?;
        api.field_lock_release(validator_set_handle)?;

        Ok(())
    }

    /// Emits a configured XRD amount ([`ConsensusManagerConfigSubstate.total_emission_xrd_per_epoch`])
    /// and distributes it across the given validator set, according to their stake.
    fn apply_validator_emissions_and_rewards<Y>(
        validator_set: ActiveValidatorSet,
        validator_statistics: Vec<ProposalStatistic>,
        config: &ConsensusManagerConfig,
        validator_rewards: &mut ValidatorRewardsSubstate,
        epoch: Epoch, // the concluded epoch, for event creation
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
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
            ) {
                validator_infos.insert(
                    TryInto::<u8>::try_into(index)
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

        let stake_sum_xrd = validator_infos
            .values()
            .map(|validator_info| validator_info.stake_xrd)
            .sum::<Decimal>();

        //======================
        // Distribute emissions
        //======================

        // calculate "how much XRD is emitted by 1 XRD staked", and later apply it evenly among validators
        // (the gains are slightly rounded down, but more fairly distributed - not affected by different rounding errors for different validators)
        let emission_per_staked_xrd = config.total_emission_xrd_per_epoch / stake_sum_xrd;
        let effective_total_emission_xrd = validator_infos
            .values()
            .map(|validator_info| validator_info.effective_stake_xrd * emission_per_staked_xrd)
            .sum::<Decimal>();

        let total_emission_xrd_bucket =
            ResourceManager(RADIX_TOKEN).mint_fungible(effective_total_emission_xrd, api)?;

        for validator_info in validator_infos.values() {
            let emission_xrd_bucket = total_emission_xrd_bucket.take(
                validator_info.effective_stake_xrd * emission_per_staked_xrd,
                api,
            )?;
            api.call_method(
                validator_info.address.as_node_id(),
                VALIDATOR_APPLY_EMISSION_IDENT,
                scrypto_encode(&ValidatorApplyEmissionInput {
                    xrd_bucket: emission_xrd_bucket,
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
        let total_individual_amount: Decimal =
            validator_rewards.proposer_rewards.values().cloned().sum();
        let reward_per_staked_xrd = (validator_rewards.rewards_vault.amount(api)?
            - total_individual_amount)
            / stake_sum_xrd;
        for (index, validator_info) in validator_infos {
            let from_self = validator_rewards
                .proposer_rewards
                .remove(&index)
                .unwrap_or_default();
            let from_pool = validator_info.effective_stake_xrd * reward_per_staked_xrd;
            let reward_amount = from_self + from_pool;
            if reward_amount.is_zero() {
                continue;
            }

            // Note that dusted xrd (due to rounding) are kept in the vault and will
            // become retrievable next time.
            let xrd_bucket = validator_rewards.rewards_vault.take(reward_amount, api)?;

            api.call_method(
                validator_info.address.as_node_id(),
                VALIDATOR_APPLY_REWARD_IDENT,
                scrypto_encode(&ValidatorApplyRewardInput { xrd_bucket, epoch }).unwrap(),
            )?;
        }

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
    ) -> Option<Self> {
        if stake_xrd.is_positive() {
            let reliability_factor = Self::to_reliability_factor(
                proposal_statistic.success_ratio(),
                min_required_reliability,
            );
            let effective_stake_xrd = stake_xrd * reliability_factor;
            Some(Self {
                address,
                stake_xrd,
                proposal_statistic,
                effective_stake_xrd,
            })
        } else {
            None
        }
    }

    /// Converts the absolute reliability measure (e.g. "0.97 uptime") into a reliability factor
    /// which directly drives the fraction of received emission (e.g. "0.25 of base emission"), by
    /// rescaling it into the allowed reliability range (e.g. "required >0.96 uptime").
    fn to_reliability_factor(reliability: Decimal, min_required_reliability: Decimal) -> Decimal {
        let reliability_reserve = reliability - min_required_reliability;
        if reliability_reserve.is_negative() {
            return Decimal::zero();
        }
        let max_allowed_unreliability = Decimal::one() - min_required_reliability;
        if max_allowed_unreliability.is_zero() {
            // special-casing the dirac delta behavior
            if reliability == Decimal::one() {
                return Decimal::one();
            } else {
                return Decimal::zero();
            }
        }
        reliability_reserve / max_allowed_unreliability
    }
}
