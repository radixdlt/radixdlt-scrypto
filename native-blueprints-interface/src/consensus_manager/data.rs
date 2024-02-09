use radix_engine_common::math::{traits::*, Decimal};
use radix_engine_common::prelude::*;
use radix_engine_macros::dec;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ConsensusManagerConfig {
    pub max_validators: u32,
    pub epoch_change_condition: EpochChangeCondition,
    pub num_unstake_epochs: u64,
    pub total_emission_xrd_per_epoch: Decimal,
    /// The proportion of proposals a validator needs to complete in an epoch to get emissions
    /// Should be between 0 and 1
    pub min_validator_reliability: Decimal,
    pub num_owner_stake_units_unlock_epochs: u64,
    pub num_fee_increase_delay_epochs: u64,

    pub validator_creation_usd_cost: Decimal,
}

impl ConsensusManagerConfig {
    pub fn with_max_validators(mut self, new_value: u32) -> Self {
        self.max_validators = new_value;
        self
    }

    pub fn with_epoch_change_condition(mut self, new_value: EpochChangeCondition) -> Self {
        self.epoch_change_condition = new_value;
        self
    }

    pub fn with_num_unstake_epochs(mut self, new_value: u64) -> Self {
        self.num_unstake_epochs = new_value;
        self
    }

    pub fn with_total_emission_xrd_per_epoch(mut self, new_value: Decimal) -> Self {
        self.total_emission_xrd_per_epoch = new_value;
        self
    }

    pub fn with_min_validator_reliability(mut self, new_value: Decimal) -> Self {
        self.min_validator_reliability = new_value;
        self
    }

    pub fn with_num_owner_stake_units_unlock_epochs(mut self, new_value: u64) -> Self {
        self.num_owner_stake_units_unlock_epochs = new_value;
        self
    }

    pub fn with_num_fee_increase_delay_epochs(mut self, new_value: u64) -> Self {
        self.num_fee_increase_delay_epochs = new_value;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
pub struct EpochChangeCondition {
    /// A minimum number of rounds that *must* happen in an epoch.
    /// The timestamp will not drive the epoch progression until at least this number of rounds is
    /// reached (i.e. if an actual number of rounds after [`duration_millis`] is less than this
    /// value, the epoch change will wait until this value is reached).
    pub min_round_count: u64,

    /// A maximum number of rounds that *can* happen in an epoch.
    /// If an actual number of rounds reaches this value before [`duration_millis`], then the
    /// timestamp no longer drives the epoch progression (i.e. the epoch change will happen right
    /// away, to prevent more than [`max_round_count`] rounds).
    pub max_round_count: u64,

    /// An "ideal" duration of an epoch, which should be applied if the number of epochs is within
    /// the `min_round_count..max_round_count` range.
    /// Note: the range exists in order to limit the amount of damage that can be done by
    /// semi-byzantine purposeful clock drift attacks.
    pub target_duration_millis: u64,
}

pub enum EpochChangeOutcome {
    NoChange,
    Change {
        next_epoch_effective_start_millis: i64,
    },
}

impl EpochChangeCondition {
    /// Determines whether this condition is met by the given actual state.
    /// See the condition's field definitions for exact rules.
    pub fn should_epoch_change(
        &self,
        effective_start: i64,
        current_time: i64,
        round: Round,
    ) -> EpochChangeOutcome {
        let epoch_duration_millis =
            // The application invariants in `check_non_decreasing_and_update_timestamps`
            // ensures that current_time > effective_start, and genesis should ensure
            // effective_start > 0.
            // This is just a sanity-check to avoid overflow if something invaraint fails.
            if current_time >= 0 && effective_start >= 0 && current_time > effective_start {
                (current_time - effective_start) as u64
            } else {
                0
            };
        if self.is_change_criterion_met(epoch_duration_millis, round) {
            // The following aims to prevent small systematic drift in the epoch length each epoch,
            // due to overheads / time noticing end of epoch.
            // If the actual epoch length is sufficiently close to the target epoch length, we just
            // pretend the effective epoch length was actually the target epoch length.
            let next_epoch_effective_start_millis =
                if self.is_actual_duration_close_to_target(epoch_duration_millis) {
                    effective_start.saturating_add_unsigned(self.target_duration_millis)
                } else {
                    current_time
                };
            EpochChangeOutcome::Change {
                next_epoch_effective_start_millis,
            }
        } else {
            EpochChangeOutcome::NoChange
        }
    }

    fn is_actual_duration_close_to_target(&self, actual_duration_millis: u64) -> bool {
        let bounds_are_compatible_with_calculation =
            actual_duration_millis >= 1000 && self.target_duration_millis >= 1000;
        if !bounds_are_compatible_with_calculation {
            // Need to avoid issues with divide by zero etc
            return false;
        }
        // NOTE: Decimal arithmetic operation safe unwrap.
        // No realistic chance to overflow.
        // 100 years in ms is less than 2^35
        let proportion_difference = (Decimal::from(actual_duration_millis)
            .checked_sub(self.target_duration_millis)
            .expect("Overflow"))
        .checked_div(self.target_duration_millis)
        .expect("Overflow");
        proportion_difference <= dec!("0.1")
    }

    fn is_change_criterion_met(&self, duration_millis: u64, round: Round) -> bool {
        if round.number() >= self.max_round_count {
            true
        } else if round.number() < self.min_round_count {
            false
        } else {
            duration_millis >= self.target_duration_millis
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct LeaderProposalHistory {
    /// The validators which were leaders of the "gap" rounds (i.e. those that were not reported to
    /// the consensus manager since the previous call; see `ConsensusManagerNextRoundInput::round`).
    /// This list will contain exactly `current_call.round - previous_call.round - 1` elements; in
    /// theory, this makes `ConsensusManagerNextRoundInput::round` field redundant (i.e. computable),
    /// but this relation can be used for an extra consistency check.
    /// The validators on this list should be penalized during emissions at the end of the current
    /// epoch.
    pub gap_round_leaders: Vec<ValidatorIndex>,

    /// The leader of the current round.
    pub current_leader: ValidatorIndex,

    /// Whether the current round was conducted in a "fallback" mode (i.e. indicating a fault
    /// of the current leader).
    /// When `true`, the `current_leader` should be penalized during emissions in the same way as
    /// `gap_round_leaders`.
    /// When `false`, the `current_leader` is considered to have made this round's proposal
    /// successfully.
    pub is_fallback: bool,
}
