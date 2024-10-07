use crate::blueprints::component::*;
use crate::blueprints::resource::*;
use crate::internal_prelude::*;
use radix_common::crypto::Secp256k1PublicKey;
use radix_common::data::manifest::model::ManifestAddressReservation;
use radix_common::math::{traits::*, Decimal};
use radix_common::prelude::ManifestBucket;
use radix_common::prelude::CONSENSUS_MANAGER_PACKAGE;
use radix_common::time::{Instant, TimeComparisonOperator};
use radix_common::types::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

pub const CONSENSUS_MANAGER_BLUEPRINT: &str = "ConsensusManager";
pub const VALIDATOR_BLUEPRINT: &str = "Validator";

define_type_marker!(Some(CONSENSUS_MANAGER_PACKAGE), ConsensusManager);
define_type_marker!(Some(CONSENSUS_MANAGER_PACKAGE), Validator);

pub const CONSENSUS_MANAGER_CREATE_IDENT: &str = "create";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ConsensusManagerCreateInput {
    pub validator_owner_token_address: GlobalAddressReservation,
    pub component_address: GlobalAddressReservation,
    pub initial_epoch: Epoch,
    pub initial_config: ConsensusManagerConfig,
    pub initial_time_ms: i64,
    pub initial_current_leader: Option<ValidatorIndex>,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ConsensusManagerCreateManifestInput {
    pub validator_owner_token_address: ManifestAddressReservation,
    pub component_address: ManifestAddressReservation,
    pub initial_epoch: Epoch,
    pub initial_config: ConsensusManagerConfig,
    pub initial_time_ms: i64,
    pub initial_current_leader: Option<ValidatorIndex>,
}

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
    pub fn mainnet_genesis() -> Self {
        Self {
            max_validators: 100,
            epoch_change_condition: EpochChangeCondition {
                min_round_count: 500,
                max_round_count: 3000,
                target_duration_millis: 300000,
            },
            num_unstake_epochs: 2016,
            total_emission_xrd_per_epoch: dec!("2853.881278538812785388"),
            min_validator_reliability: Decimal::one(),
            num_owner_stake_units_unlock_epochs: 8064,
            num_fee_increase_delay_epochs: 4032,
            validator_creation_usd_cost: dec!(1000),
        }
    }

    pub fn test_default() -> Self {
        ConsensusManagerConfig {
            max_validators: 10,
            epoch_change_condition: EpochChangeCondition {
                min_round_count: 1,
                max_round_count: 1,
                target_duration_millis: 0,
            },
            num_unstake_epochs: 1,
            total_emission_xrd_per_epoch: Decimal::one(),
            min_validator_reliability: Decimal::one(),
            num_owner_stake_units_unlock_epochs: 2,
            num_fee_increase_delay_epochs: 1,
            validator_creation_usd_cost: dec!(100),
        }
    }

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

pub type ConsensusManagerCreateOutput = ();

pub const CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT: &str = "get_current_epoch";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerGetCurrentEpochInput;

pub type ConsensusManagerGetCurrentEpochManifestInput = ConsensusManagerGetCurrentEpochInput;

pub type ConsensusManagerGetCurrentEpochOutput = Epoch;

pub const CONSENSUS_MANAGER_START_IDENT: &str = "start";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerStartInput {}

pub type ConsensusManagerStartManifestInput = ConsensusManagerStartInput;

pub type ConsensusManagerStartOutput = ();

#[derive(Copy, Clone, Debug, Eq, PartialEq, Sbor)]
#[sbor(type_name = "TimePrecision")]
pub enum TimePrecisionV1 {
    Minute,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Sbor)]
#[sbor(type_name = "TimePrecision")]
pub enum TimePrecisionV2 {
    Minute,
    Second,
}

pub type TimePrecision = TimePrecisionV2;

pub const CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT: &str = "get_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
#[sbor(type_name = "ConsensusManagerGetCurrentTimeInput")]
pub struct ConsensusManagerGetCurrentTimeInputV1 {
    pub precision: TimePrecisionV1,
}

pub type ConsensusManagerGetCurrentTimeManifestInputV1 = ConsensusManagerGetCurrentTimeInputV1;

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
#[sbor(type_name = "ConsensusManagerGetCurrentTimeInput")]
pub struct ConsensusManagerGetCurrentTimeInputV2 {
    pub precision: TimePrecisionV2,
}

pub type ConsensusManagerGetCurrentTimeManifestInputV2 = ConsensusManagerGetCurrentTimeInputV2;

pub type ConsensusManagerGetCurrentTimeOutput = Instant;

pub const CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT: &str = "compare_current_time";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
#[sbor(type_name = "ConsensusManagerCompareCurrentTimeInput")]
pub struct ConsensusManagerCompareCurrentTimeInputV1 {
    pub instant: Instant,
    pub precision: TimePrecisionV1,
    pub operator: TimeComparisonOperator,
}

pub type ConsensusManagerCompareCurrentTimeManifestInputV1 =
    ConsensusManagerCompareCurrentTimeInputV1;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
#[sbor(type_name = "ConsensusManagerCompareCurrentTimeInput")]
pub struct ConsensusManagerCompareCurrentTimeInputV2 {
    pub instant: Instant,
    pub precision: TimePrecisionV2,
    pub operator: TimeComparisonOperator,
}

pub type ConsensusManagerCompareCurrentTimeManifestInputV2 =
    ConsensusManagerCompareCurrentTimeInputV2;

pub type ConsensusManagerCompareCurrentTimeOutput = bool;

pub const CONSENSUS_MANAGER_NEXT_ROUND_IDENT: &str = "next_round";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerNextRoundInput {
    /// Current round number.
    /// Please note that in case of liveness breaks, this number may be different than previous
    /// reported `round + 1`. Such gaps are considered "round leader's fault" and are penalized
    /// on emission, according to leader reliability statistics (see `LeaderProposalHistory`).
    pub round: Round,

    /// Current millisecond timestamp of the proposer.
    pub proposer_timestamp_ms: i64,

    /// A captured history of leader proposal reliability since the previously reported round.
    // TODO(post-babylon): we should change the approach here, so that the Engine drives the
    // leader rotation, and the Node only informs it on round success/fallback/miss (in order to
    // avoid certain byzantine quorum behaviors). The entire `leader_proposal_history` information
    // will then no longer be required.
    pub leader_proposal_history: LeaderProposalHistory,
}

pub type ConsensusManagerNextRoundManifestInput = ConsensusManagerNextRoundInput;

impl ConsensusManagerNextRoundInput {
    /// Creates a "next round" input for a regular (happy-path, in terms of consensus) round
    /// progression, i.e. no missed proposals, no fallback rounds.
    /// Please note that the current round's number passed here should be an immediate successor of
    /// the previously reported round.
    pub fn successful(
        current_round: Round,
        current_leader: ValidatorIndex,
        current_timestamp_ms: i64,
    ) -> Self {
        Self {
            round: current_round,
            proposer_timestamp_ms: current_timestamp_ms,
            leader_proposal_history: LeaderProposalHistory {
                gap_round_leaders: Vec::new(),
                current_leader,
                is_fallback: false,
            },
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

pub type ConsensusManagerNextRoundOutput = ();

pub const CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT: &str = "create_validator";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ConsensusManagerCreateValidatorInput {
    pub key: Secp256k1PublicKey,
    pub fee_factor: Decimal,
    pub xrd_payment: Bucket,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ConsensusManagerCreateValidatorManifestInput {
    pub key: Secp256k1PublicKey,
    pub fee_factor: Decimal,
    pub xrd_payment: ManifestBucket,
}

pub type ConsensusManagerCreateValidatorOutput = (Global<ValidatorMarker>, Bucket, Bucket);

pub const VALIDATOR_REGISTER_IDENT: &str = "register";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorRegisterInput {}

pub type ValidatorRegisterManifestInput = ValidatorRegisterInput;

pub type ValidatorRegisterOutput = ();

pub const VALIDATOR_UNREGISTER_IDENT: &str = "unregister";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUnregisterInput {}

pub type ValidatorUnregisterManifestInput = ValidatorUnregisterInput;

pub type ValidatorUnregisterOutput = ();

pub const VALIDATOR_STAKE_AS_OWNER_IDENT: &str = "stake_as_owner";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorStakeAsOwnerInput {
    pub stake: Bucket,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ValidatorStakeAsOwnerManifestInput {
    pub stake: ManifestBucket,
}

pub type ValidatorStakeAsOwnerOutput = Bucket;

pub const VALIDATOR_STAKE_IDENT: &str = "stake";
#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorStakeInput {
    pub stake: Bucket,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ValidatorStakeManifestInput {
    pub stake: ManifestBucket,
}

pub type ValidatorStakeOutput = Bucket;

pub const VALIDATOR_UNSTAKE_IDENT: &str = "unstake";
#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorUnstakeInput {
    pub stake_unit_bucket: Bucket,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ValidatorUnstakeManifestInput {
    pub stake_unit_bucket: ManifestBucket,
}

pub type ValidatorUnstakeOutput = Bucket;

pub const VALIDATOR_CLAIM_XRD_IDENT: &str = "claim_xrd";
#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorClaimXrdInput {
    pub bucket: Bucket,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ValidatorClaimXrdManifestInput {
    pub bucket: ManifestBucket,
}

pub type ValidatorClaimXrdOutput = Bucket;

pub const VALIDATOR_UPDATE_KEY_IDENT: &str = "update_key";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ValidatorUpdateKeyInput {
    pub key: Secp256k1PublicKey,
}

pub type ValidatorUpdateKeyManifestInput = ValidatorUpdateKeyInput;

pub type ValidatorUpdateKeyOutput = ();

pub const VALIDATOR_UPDATE_FEE_IDENT: &str = "update_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ValidatorUpdateFeeInput {
    /// A fraction of the effective emission amount which gets transferred to the validator's owner.
    /// Must be within `[0.0, 1.0]`.
    pub new_fee_factor: Decimal,
}

pub type ValidatorUpdateFeeManifestInput = ValidatorUpdateFeeInput;

pub type ValidatorUpdateFeeOutput = ();

pub const VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT: &str = "update_accept_delegated_stake";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUpdateAcceptDelegatedStakeInput {
    pub accept_delegated_stake: bool,
}

pub type ValidatorUpdateAcceptDelegatedStakeManifestInput =
    ValidatorUpdateAcceptDelegatedStakeInput;

pub type ValidatorUpdateAcceptDelegatedStakeOutput = ();

pub const VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT: &str = "accepts_delegated_stake";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorAcceptsDelegatedStakeInput {}

pub type ValidatorAcceptsDelegatedStakeManifestInput = ValidatorAcceptsDelegatedStakeInput;

pub type ValidatorAcceptsDelegatedStakeOutput = bool;

pub const VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT: &str = "total_stake_xrd_amount";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorTotalStakeXrdAmountInput {}

pub type ValidatorTotalStakeXrdAmountManifestInput = ValidatorTotalStakeXrdAmountInput;

pub type ValidatorTotalStakeXrdAmountOutput = Decimal;

pub const VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT: &str = "total_stake_unit_supply";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorTotalStakeUnitSupplyInput {}

pub type ValidatorTotalStakeUnitSupplyManifestInput = ValidatorTotalStakeUnitSupplyInput;

pub type ValidatorTotalStakeUnitSupplyOutput = Decimal;

pub const VALIDATOR_GET_REDEMPTION_VALUE_IDENT: &str = "get_redemption_value";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ValidatorGetRedemptionValueInput {
    pub amount_of_stake_units: Decimal,
}

pub type ValidatorGetRedemptionValueManifestInput = ValidatorGetRedemptionValueInput;

pub type ValidatorGetRedemptionValueOutput = Decimal;

pub const VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS_IDENT: &str =
    "signal_protocol_update_readiness";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorSignalProtocolUpdateReadinessInput {
    pub vote: String,
}

pub type ValidatorSignalProtocolUpdateReadinessManifestInput =
    ValidatorSignalProtocolUpdateReadinessInput;

pub type ValidatorSignalProtocolUpdateReadinessOutput = ();

pub const VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT: &str = "get_protocol_update_readiness";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorGetProtocolUpdateReadinessInput {}

pub type ValidatorGetProtocolUpdateReadinessManifestInput =
    ValidatorGetProtocolUpdateReadinessInput;

pub type ValidatorGetProtocolUpdateReadinessOutput = Option<String>;

pub const VALIDATOR_APPLY_EMISSION_IDENT: &str = "apply_emission";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorApplyEmissionInput {
    /// A bucket with the emitted XRDs for this validator.
    /// The validator should subtract the configured fee from this amount.
    pub xrd_bucket: Bucket,
    /// The *concluded* epoch's number. Informational-only.
    pub epoch: Epoch,
    /// A number of proposals successfully made by this validator during the emission period.
    pub proposals_made: u64,
    /// A number of proposals missed by this validator during the emission period.
    pub proposals_missed: u64,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ValidatorApplyEmissionManifestInput {
    /// A bucket with the emitted XRDs for this validator.
    /// The validator should subtract the configured fee from this amount.
    pub xrd_bucket: ManifestBucket,
    /// The *concluded* epoch's number. Informational-only.
    pub epoch: Epoch,
    /// A number of proposals successfully made by this validator during the emission period.
    pub proposals_made: u64,
    /// A number of proposals missed by this validator during the emission period.
    pub proposals_missed: u64,
}

pub type ValidatorApplyEmissionOutput = ();

pub const VALIDATOR_APPLY_REWARD_IDENT: &str = "apply_reward";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorApplyRewardInput {
    /// A bucket with the rewarded XRDs (from transaction fees) for this validator.
    pub xrd_bucket: Bucket,
    /// The *concluded* epoch's number. Informational-only.
    pub epoch: Epoch,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ValidatorApplyRewardManifestInput {
    /// A bucket with the rewarded XRDs (from transaction fees) for this validator.
    pub xrd_bucket: ManifestBucket,
    /// The *concluded* epoch's number. Informational-only.
    pub epoch: Epoch,
}

pub type ValidatorApplyRewardOutput = ();

pub const VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT: &str = "lock_owner_stake_units";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorLockOwnerStakeUnitsInput {
    pub stake_unit_bucket: Bucket,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ValidatorLockOwnerStakeUnitsManifestInput {
    pub stake_unit_bucket: ManifestBucket,
}

pub type ValidatorLockOwnerStakeUnitsOutput = ();

pub const VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT: &str = "start_unlock_owner_stake_units";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ValidatorStartUnlockOwnerStakeUnitsInput {
    pub requested_stake_unit_amount: Decimal,
}

pub type ValidatorStartUnlockOwnerStakeUnitsManifestInput =
    ValidatorStartUnlockOwnerStakeUnitsInput;

pub type ValidatorStartUnlockOwnerStakeUnitsOutput = ();

pub const VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT: &str = "finish_unlock_owner_stake_units";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ValidatorFinishUnlockOwnerStakeUnitsInput {}

pub type ValidatorFinishUnlockOwnerStakeUnitsManifestInput =
    ValidatorFinishUnlockOwnerStakeUnitsInput;

pub type ValidatorFinishUnlockOwnerStakeUnitsOutput = Bucket;
