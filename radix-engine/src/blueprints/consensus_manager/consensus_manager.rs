use super::{EpochChangeEvent, RoundChangeEvent, ValidatorCreator};
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::{NativeBucket, ResourceManager};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientApi, CollectionIndex, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;

const SECONDS_TO_MS_FACTOR: i64 = 1000;
const MINUTES_TO_SECONDS_FACTOR: i64 = 60;
const MINUTES_TO_MS_FACTOR: i64 = SECONDS_TO_MS_FACTOR * MINUTES_TO_SECONDS_FACTOR;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ConsensusManagerConfigSubstate {
    pub max_validators: u32,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
    pub total_emission_xrd_per_epoch: Decimal,
    pub min_validator_reliability: Decimal,
    pub num_owner_stake_units_unlock_epochs: u64,
    pub num_fee_increase_delay_epochs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ConsensusManagerSubstate {
    pub epoch: u64,
    pub round: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, ScryptoSbor)]
pub struct Validator {
    pub key: EcdsaSecp256k1PublicKey,
    pub stake: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct CurrentValidatorSetSubstate {
    pub validator_set: BTreeMap<ComponentAddress, Validator>,
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
                    ConsensusManagerError::InvalidaValidatorIndex {
                        index: validator_index as usize,
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

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub enum ConsensusManagerError {
    InvalidRoundUpdate {
        from: u64,
        to: u64,
    },
    InconsistentGapRounds {
        gap_rounds: u64,
        progressed_rounds: u64,
    },
    InvalidaValidatorIndex {
        index: usize,
        count: usize,
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
        validator_token_address: [u8; NodeId::LENGTH], // TODO: Clean this up
        component_address: [u8; NodeId::LENGTH],       // TODO: Clean this up
        initial_epoch: u64,
        initial_configuration: ConsensusManagerInitialConfiguration,
        initial_time_ms: i64,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let address = ComponentAddress::new_or_panic(component_address);

        {
            let metadata: BTreeMap<String, String> = BTreeMap::new();
            let mut access_rules = BTreeMap::new();

            // TODO: remove mint and premint all tokens
            {
                let global_id =
                    NonFungibleGlobalId::package_of_direct_caller_badge(CONSENSUS_MANAGER_PACKAGE);
                access_rules.insert(Mint, (rule!(require(global_id)), rule!(deny_all)));
            }

            access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));

            ResourceManager::new_non_fungible_with_address::<(), Y, RuntimeError>(
                NonFungibleIdType::UUID,
                metadata,
                access_rules,
                validator_token_address,
                api,
            )?;
        };

        let consensus_manager_id = {
            let config = ConsensusManagerConfigSubstate {
                max_validators: initial_configuration.max_validators,
                rounds_per_epoch: initial_configuration.rounds_per_epoch,
                num_unstake_epochs: initial_configuration.num_unstake_epochs,
                total_emission_xrd_per_epoch: initial_configuration.total_emission_xrd_per_epoch,
                min_validator_reliability: initial_configuration.min_validator_reliability,
                num_owner_stake_units_unlock_epochs: initial_configuration
                    .num_owner_stake_units_unlock_epochs,
                num_fee_increase_delay_epochs: initial_configuration.num_fee_increase_delay_epochs,
            };
            let consensus_manager = ConsensusManagerSubstate {
                epoch: initial_epoch,
                round: 0,
            };
            let current_validator_set = CurrentValidatorSetSubstate {
                validator_set: BTreeMap::new(),
            };
            let current_proposal_statistic = CurrentProposalStatisticSubstate {
                validator_statistics: Vec::new(),
            };

            api.new_simple_object(
                CONSENSUS_MANAGER_BLUEPRINT,
                vec![
                    scrypto_encode(&config).unwrap(),
                    scrypto_encode(&consensus_manager).unwrap(),
                    scrypto_encode(&current_validator_set).unwrap(),
                    scrypto_encode(&current_proposal_statistic).unwrap(),
                    scrypto_encode(&Self::round_to_minutes_ms(initial_time_ms)).unwrap(),
                    scrypto_encode(&initial_time_ms).unwrap(),
                ],
            )?
        };

        let mut authority_rules = AuthorityRules::new();
        authority_rules.set_main_authority_rule(
            CONSENSUS_MANAGER_START_IDENT,
            rule!(require(package_of_direct_caller(CONSENSUS_MANAGER_PACKAGE))),
            rule!(require(package_of_direct_caller(CONSENSUS_MANAGER_PACKAGE))),
        );
        authority_rules.set_fixed_main_authority_rule(
            CONSENSUS_MANAGER_SET_CURRENT_TIME_IDENT,
            rule!(require(AuthAddresses::validator_role())),
        );
        authority_rules.set_fixed_main_authority_rule(
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
            rule!(require(AuthAddresses::validator_role())),
        );
        authority_rules.set_fixed_main_authority_rule(
            CONSENSUS_MANAGER_SET_EPOCH_IDENT,
            rule!(require(AuthAddresses::system_role())), // Set epoch only used for debugging
        );

        let access_rules = AccessRules::create(
            authority_rules,
            btreemap!(
                VALIDATOR_BLUEPRINT.to_string() => AuthorityRules::new()
            ),
            api,
        )?
        .0;
        let metadata = Metadata::create(api)?;
        let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

        api.globalize_with_address(
            btreemap!(
                ObjectModuleId::Main => consensus_manager_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            address.into(),
        )?;

        Ok(())
    }

    pub(crate) fn get_current_epoch<Y>(api: &mut Y) -> Result<u64, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::ConsensusManager.into(),
            LockFlags::read_only(),
        )?;

        let consensus_manager: ConsensusManagerSubstate = api.field_lock_read_typed(handle)?;

        Ok(consensus_manager.epoch)
    }

    pub(crate) fn start<Y>(receiver: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let config_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::Config.into(),
            LockFlags::read_only(),
        )?;
        let config: ConsensusManagerConfigSubstate = api.field_lock_read_typed(config_handle)?;

        let mgr_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::ConsensusManager.into(),
            LockFlags::read_only(),
        )?;
        let mgr: ConsensusManagerSubstate = api.field_lock_read_typed(mgr_handle)?;

        Self::epoch_change(mgr.epoch, &config, api)?;

        let access_rules = AttachedAccessRules(*receiver);
        access_rules.set_authority_rule_and_mutability(
            AuthorityKey::main("start"),
            AccessRule::DenyAll,
            AccessRule::DenyAll,
            api,
        )?;

        Ok(())
    }

    pub(crate) fn set_current_time<Y>(current_time_ms: i64, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::CurrentTime.into(),
            LockFlags::MUTABLE,
        )?;
        api.field_lock_write_typed(handle, &current_time_ms)?;
        api.field_lock_release(handle)?;

        let current_time_rounded_to_minutes_ms = Self::round_to_minutes_ms(current_time_ms);
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::CurrentTimeRoundedToMinutes.into(),
            LockFlags::MUTABLE,
        )?;
        api.field_lock_write_typed(handle, &current_time_rounded_to_minutes_ms)?;
        api.field_lock_release(handle)?;

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
                let handle = api.actor_lock_field(
                    OBJECT_HANDLE_SELF,
                    ConsensusManagerField::CurrentTimeRoundedToMinutes.into(),
                    LockFlags::read_only(),
                )?;
                let current_time_rounded_to_minutes_ms: i64 = api.field_lock_read_typed(handle)?;
                api.field_lock_release(handle)?;
                let instant = Self::instant_from_millis(current_time_rounded_to_minutes_ms);
                Ok(instant)
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
                let handle = api.actor_lock_field(
                    OBJECT_HANDLE_SELF,
                    ConsensusManagerField::CurrentTimeRoundedToMinutes.into(),
                    LockFlags::read_only(),
                )?;
                let current_time_rounded_to_minutes_ms: i64 = api.field_lock_read_typed(handle)?;
                api.field_lock_release(handle)?;

                let other_time_ms = Self::instant_to_millis(other_arbitrary_precision_instant);
                let other_time_rounded_to_minutes_ms = Self::round_to_minutes_ms(other_time_ms);

                let current_instant = Self::instant_from_millis(current_time_rounded_to_minutes_ms);
                let other_instant = Self::instant_from_millis(other_time_rounded_to_minutes_ms);
                let result = current_instant.compare(other_instant, operator);
                Ok(result)
            }
        }
    }

    fn round_to_minutes_ms(time_ms: i64) -> i64 {
        (time_ms / MINUTES_TO_MS_FACTOR) * MINUTES_TO_MS_FACTOR
    }

    fn instant_from_millis(time_ms: i64) -> Instant {
        Instant::new(time_ms / SECONDS_TO_MS_FACTOR)
    }

    fn instant_to_millis(instant: Instant) -> i64 {
        instant.seconds_since_unix_epoch * SECONDS_TO_MS_FACTOR
    }

    pub(crate) fn next_round<Y>(
        round: u64,
        proposal_history: LeaderProposalHistory,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let config_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::Config.into(),
            LockFlags::read_only(),
        )?;
        let config: ConsensusManagerConfigSubstate = api.field_lock_read_typed(config_handle)?;
        let mgr_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::ConsensusManager.into(),
            LockFlags::MUTABLE,
        )?;
        let mut consensus_manager: ConsensusManagerSubstate =
            api.field_lock_read_typed(mgr_handle)?;

        let progressed_rounds = round as i128 - consensus_manager.round as i128;
        if progressed_rounds <= 0 {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ConsensusManagerError(
                    ConsensusManagerError::InvalidRoundUpdate {
                        from: consensus_manager.round,
                        to: round,
                    },
                ),
            ));
        }

        Self::update_proposal_statistics(progressed_rounds as u64, proposal_history, api)?;

        if round >= config.rounds_per_epoch {
            let next_epoch = consensus_manager.epoch + 1;
            Self::epoch_change(next_epoch, &config, api)?;
            consensus_manager.epoch = next_epoch;
            consensus_manager.round = 0;
        } else {
            Runtime::emit_event(api, RoundChangeEvent { round })?;
            consensus_manager.round = round;
        }

        api.field_lock_write_typed(mgr_handle, &consensus_manager)?;
        api.field_lock_release(mgr_handle)?;

        Ok(())
    }

    pub(crate) fn set_epoch<Y>(epoch: u64, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::ConsensusManager.into(),
            LockFlags::MUTABLE,
        )?;

        let mut consensus_manager: ConsensusManagerSubstate = api.field_lock_read_typed(handle)?;
        consensus_manager.epoch = epoch;
        api.field_lock_write_typed(handle, &consensus_manager)?;

        Ok(())
    }

    pub(crate) fn create_validator<Y>(
        key: EcdsaSecp256k1PublicKey,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let (validator_address, owner_token_bucket) = ValidatorCreator::create(key, false, api)?;

        Ok((validator_address, owner_token_bucket))
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
                        gap_rounds: proposal_history.gap_round_leaders.len() as u64,
                        progressed_rounds,
                    },
                ),
            ));
        }

        let statistic_handle = api.actor_lock_field(
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
        next_epoch: u64,
        config: &ConsensusManagerConfigSubstate,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // read previous validator set
        let validator_set_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::CurrentValidatorSet.into(),
            LockFlags::MUTABLE,
        )?;
        let mut validator_set_substate: CurrentValidatorSetSubstate =
            api.field_lock_read_typed(validator_set_handle)?;
        let previous_validator_set = validator_set_substate.validator_set;

        // read previous validator statistics
        let statistic_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ConsensusManagerField::CurrentProposalStatistic.into(),
            LockFlags::MUTABLE,
        )?;
        let mut statistic_substate: CurrentProposalStatisticSubstate =
            api.field_lock_read_typed(statistic_handle)?;
        let previous_statistics = statistic_substate.validator_statistics;

        // apply emissions
        Self::apply_validator_emissions(
            previous_validator_set,
            previous_statistics,
            config,
            next_epoch - 1,
            api,
        )?;

        // select next validator set
        let registered_validators: Vec<EpochRegisteredValidatorByStakeEntry> = api
            .actor_sorted_index_scan_typed(
                OBJECT_HANDLE_SELF,
                CONSENSUS_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX,
                config.max_validators,
            )?;
        let next_validator_set: BTreeMap<ComponentAddress, Validator> = registered_validators
            .into_iter()
            .map(|entry| (entry.component_address, entry.validator))
            .collect();

        // emit epoch change event
        Runtime::emit_event(
            api,
            EpochChangeEvent {
                epoch: next_epoch,
                validators: next_validator_set.clone(),
            },
        )?;

        // write zeroed statistics of next validators
        statistic_substate.validator_statistics = (0..next_validator_set.len())
            .map(|_index| ProposalStatistic::default())
            .collect();
        api.field_lock_write_typed(statistic_handle, statistic_substate)?;
        api.field_lock_release(statistic_handle)?;

        // write next validator set
        validator_set_substate.validator_set = next_validator_set;
        api.field_lock_write_typed(validator_set_handle, validator_set_substate)?;
        api.field_lock_release(validator_set_handle)?;

        Ok(())
    }

    /// Emits a configured XRD amount ([`ConsensusManagerConfigSubstate.total_emission_xrd_per_epoch`])
    /// and distributes it across the given validator set, according to their stake.
    fn apply_validator_emissions<Y>(
        validator_set: BTreeMap<ComponentAddress, Validator>,
        validator_statistics: Vec<ProposalStatistic>,
        config: &ConsensusManagerConfigSubstate,
        epoch: u64, // the concluded epoch, for event creation
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let validator_emissions = validator_set
            .into_iter()
            .zip(validator_statistics)
            .filter_map(|((address, validator), statistic)| {
                ValidatorEmission::create_if_applicable(
                    address,
                    validator.stake,
                    statistic,
                    config.min_validator_reliability,
                )
            })
            .collect::<Vec<_>>();

        if validator_emissions.is_empty() {
            return Ok(());
        }

        let stake_sum_xrd = validator_emissions
            .iter()
            .map(|validator_emission| validator_emission.stake_xrd)
            .sum::<Decimal>();
        // calculate "how much XRD is emitted by 1 XRD staked", and later apply it evenly among validators
        // (the gains are slightly rounded down, but more fairly distributed - not affected by different rounding errors for different validators)
        let emission_per_staked_xrd = config.total_emission_xrd_per_epoch / stake_sum_xrd;
        let effective_total_emission_xrd = validator_emissions
            .iter()
            .map(|validator_emission| {
                validator_emission.effective_stake_xrd * emission_per_staked_xrd
            })
            .sum::<Decimal>();

        let total_emission_xrd_bucket =
            ResourceManager(RADIX_TOKEN).mint_fungible(effective_total_emission_xrd, api)?;

        for validator_emission in validator_emissions {
            let emission_xrd_bucket = total_emission_xrd_bucket.take(
                validator_emission.effective_stake_xrd * emission_per_staked_xrd,
                api,
            )?;
            api.call_method(
                validator_emission.address.as_node_id(),
                VALIDATOR_APPLY_EMISSION_IDENT,
                scrypto_encode(&ValidatorApplyEmissionInput {
                    xrd_bucket: emission_xrd_bucket,
                    epoch,
                    proposals_made: validator_emission.proposal_statistic.made,
                    proposals_missed: validator_emission.proposal_statistic.missed,
                })
                .unwrap(),
            )?;
        }
        total_emission_xrd_bucket.drop_empty(api)?;

        Ok(())
    }
}

#[derive(Debug)]
struct ValidatorEmission {
    pub address: ComponentAddress,
    pub stake_xrd: Decimal,
    pub effective_stake_xrd: Decimal,
    pub proposal_statistic: ProposalStatistic, // needed only for passing the information to event
}

impl ValidatorEmission {
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
