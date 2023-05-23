use crate::api::actor_sorted_index_api::SortedKey;
use crate::blueprints::resource::*;
use crate::*;
use radix_engine_common::time::{Instant, TimeComparisonOperator};
use radix_engine_common::types::*;
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::math::Decimal;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

pub const CONSENSUS_MANAGER_BLUEPRINT: &str = "ConsensusManager";
pub const VALIDATOR_BLUEPRINT: &str = "Validator";

pub const CONSENSUS_MANAGER_CREATE_IDENT: &str = "create";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ConsensusManagerCreateInput {
    pub validator_owner_token: [u8; NodeId::LENGTH], // TODO: Clean this up
    pub component_address: [u8; NodeId::LENGTH],     // TODO: Clean this up
    pub initial_epoch: u64,
    pub initial_configuration: ConsensusManagerInitialConfiguration,
    pub initial_time_ms: i64,
}

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ConsensusManagerInitialConfiguration {
    pub max_validators: u32,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
    pub total_emission_xrd_per_epoch: Decimal,
    pub min_validator_reliability: Decimal,
    pub num_owner_stake_units_unlock_epochs: u64,
    pub num_fee_increase_delay_epochs: u64,
}

impl ConsensusManagerInitialConfiguration {
    pub fn with_max_validators(mut self, new_value: u32) -> Self {
        self.max_validators = new_value;
        self
    }

    pub fn with_rounds_per_epoch(mut self, new_value: u64) -> Self {
        self.rounds_per_epoch = new_value;
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

pub type ConsensusManagerCreateOutput = ();

pub const CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT: &str = "get_current_epoch";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerGetCurrentEpochInput;

pub type ConsensusManagerGetCurrentEpochOutput = u64;

pub const CONSENSUS_MANAGER_SET_EPOCH_IDENT: &str = "set_epoch";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerSetEpochInput {
    pub epoch: u64,
}

pub type ConsensusManagerSetEpochOutput = ();

pub const CONSENSUS_MANAGER_START_IDENT: &str = "start";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerStartInput {}

pub type ConsensusManagerStartOutput = ();

#[derive(Sbor, Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimePrecision {
    Minute,
}

pub const CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT: &str = "get_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerGetCurrentTimeInput {
    pub precision: TimePrecision,
}

pub type ConsensusManagerGetCurrentTimeOutput = Instant;

pub const CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT: &str = "compare_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerCompareCurrentTimeInput {
    pub instant: Instant,
    pub precision: TimePrecision,
    pub operator: TimeComparisonOperator,
}

pub type ConsensusManagerCompareCurrentTimeOutput = bool;

pub const CONSENSUS_MANAGER_SET_CURRENT_TIME_IDENT: &str = "set_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerSetCurrentTimeInput {
    pub current_time_ms: i64,
}

pub type ConsensusManagerSetCurrentTimeOutput = ();

pub const CONSENSUS_MANAGER_NEXT_ROUND_IDENT: &str = "next_round";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerNextRoundInput {
    /// Current round number.
    /// Please note that in case of liveness breaks, this number may be different than previous
    /// reported `round + 1`. Such gaps are considered "round leader's fault" and are penalized
    /// on emission, according to leader reliability statistics (see `LeaderProposalHistory`).
    pub round: u64,

    /// A captured history of leader proposal reliability since the previously reported round.
    // TODO(post-babylon): we should change the approach here, so that the Engine drives the
    // leader rotation, and the Node only informs it on round success/fallback/miss (in order to
    // avoid certain byzantine quorum behaviors). The entire `leader_proposal_history` information
    // will then no longer be required.
    pub leader_proposal_history: LeaderProposalHistory,
}

impl ConsensusManagerNextRoundInput {
    /// Creates a "next round" input for a regular (happy-path, in terms of consensus) round
    /// progression, i.e. no missed proposals, no fallback rounds.
    /// Please note that the current round's number passed here should be an immediate successor of
    /// the previously reported round.
    pub fn successful(current_round: u64, current_leader: ValidatorIndex) -> Self {
        Self {
            round: current_round,
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

/// An index of a specific validator within the current validator set.
/// To be exact: a `ValidatorIndex` equal to `k` references the `k-th` element returned by the
/// iterator of `BTreeMap<ComponentAddress, Validator>`.
/// This uniquely identifies the validator, while being shorter than `ComponentAddress` (we do care
/// about the constant factor of the space taken by `LeaderProposalHistory` under prolonged liveness
/// break scenarios).
pub type ValidatorIndex = u8;

pub type ConsensusManagerNextRoundOutput = ();

pub const CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT: &str = "create_validator";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ConsensusManagerCreateValidatorInput {
    pub key: EcdsaSecp256k1PublicKey,
}

pub type ConsensusManagerCreateValidatorOutput = (ComponentAddress, Bucket);

pub const CONSENSUS_MANAGER_UPDATE_VALIDATOR_IDENT: &str = "update_validator";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub enum UpdateSecondaryIndex {
    Create {
        index_key: SortedKey,
        primary: ComponentAddress,
        key: EcdsaSecp256k1PublicKey,
        stake: Decimal,
    },
    UpdateStake {
        index_key: SortedKey,
        new_index_key: SortedKey,
        new_stake_amount: Decimal,
    },
    UpdatePublicKey {
        index_key: SortedKey,
        key: EcdsaSecp256k1PublicKey,
    },
    Remove {
        index_key: SortedKey,
    },
}

pub const VALIDATOR_REGISTER_IDENT: &str = "register";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorRegisterInput {}

pub type ValidatorRegisterOutput = ();

pub const VALIDATOR_UNREGISTER_IDENT: &str = "unregister";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUnregisterInput {}

pub type ValidatorUnregisterOutput = ();

pub const VALIDATOR_STAKE_IDENT: &str = "stake";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorStakeInput {
    pub stake: Bucket,
}

pub type ValidatorStakeOutput = Bucket;

pub const VALIDATOR_UNSTAKE_IDENT: &str = "unstake";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorUnstakeInput {
    pub stake_unit_bucket: Bucket,
}

pub type ValidatorUnstakeOutput = Bucket;

pub const VALIDATOR_CLAIM_XRD_IDENT: &str = "claim_xrd";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorClaimXrdInput {
    pub bucket: Bucket,
}

pub type ValidatorClaimXrdOutput = Bucket;

pub const VALIDATOR_UPDATE_KEY_IDENT: &str = "update_key";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUpdateKeyInput {
    pub key: EcdsaSecp256k1PublicKey,
}

pub type ValidatorUpdateKeyOutput = ();

pub const VALIDATOR_UPDATE_FEE_IDENT: &str = "update_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorUpdateFeeInput {
    /// A fraction of the effective emission amount which gets transferred to the validator's owner.
    /// Must be within `[0.0, 1.0]`.
    pub new_fee_factor: Decimal,
}

pub type ValidatorUpdateFeeOutput = ();

pub const VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT: &str = "update_accept_delegated_stake";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUpdateAcceptDelegatedStakeInput {
    pub accept_delegated_stake: bool,
}

pub type ValidatorUpdateAcceptDelegatedStakeOutput = ();

pub const VALIDATOR_APPLY_EMISSION_IDENT: &str = "apply_emission";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorApplyEmissionInput {
    /// A bucket with the emitted XRDs for this validator.
    /// The validator should subtract the configured fee from this amount.
    pub xrd_bucket: Bucket,
    /// The *concluded* epoch's number. Informational-only.
    pub epoch: u64,
    /// A number of proposals successfully made by this validator during the emission period.
    pub proposals_made: u64,
    /// A number of proposals missed by this validator during the emission period.
    pub proposals_missed: u64,
}

pub type ValidatorApplyEmissionOutput = ();

pub const VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT: &str = "lock_owner_stake_units";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorLockOwnerStakeUnitsInput {
    pub stake_unit_bucket: Bucket,
}

pub type ValidatorLockOwnerStakeUnitsOutput = ();

pub const VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT: &str = "start_unlock_owner_stake_units";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorStartUnlockOwnerStakeUnitsInput {
    pub requested_stake_unit_amount: Decimal,
}

pub type ValidatorStartUnlockOwnerStakeUnitsOutput = Bucket;

pub const VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT: &str = "finish_unlock_owner_stake_units";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorFinishUnlockOwnerStakeUnitsInput {}

pub type ValidatorFinishUnlockOwnerStakeUnitsOutput = Bucket;
