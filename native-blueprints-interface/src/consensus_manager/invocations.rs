use super::*;
use crate::resource::*;
use radix_engine_common::crypto::Secp256k1PublicKey;
use radix_engine_common::data::manifest::model::ManifestAddressReservation;
use radix_engine_common::define_type_info_marker;
use radix_engine_common::prelude::ManifestBucket;
use radix_engine_common::prelude::CONSENSUS_MANAGER_PACKAGE;
use radix_engine_common::prelude::*;
use radix_engine_common::time::{Instant, TimeComparisonOperator};
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

pub const CONSENSUS_MANAGER_BLUEPRINT: &str = "ConsensusManager";
pub const VALIDATOR_BLUEPRINT: &str = "Validator";

define_type_info_marker!(Some(CONSENSUS_MANAGER_PACKAGE), ConsensusManager);
define_type_info_marker!(Some(CONSENSUS_MANAGER_PACKAGE), Validator);

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

pub type ConsensusManagerCreateOutput = ();

pub const CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT: &str = "get_current_epoch";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerGetCurrentEpochInput;

pub type ConsensusManagerGetCurrentEpochOutput = Epoch;

pub const CONSENSUS_MANAGER_START_IDENT: &str = "start";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ConsensusManagerStartInput {}

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

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
#[sbor(type_name = "ConsensusManagerGetCurrentTimeInput")]
pub struct ConsensusManagerGetCurrentTimeInputV2 {
    pub precision: TimePrecisionV2,
}

pub type ConsensusManagerGetCurrentTimeOutput = Instant;

pub const CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT: &str = "compare_current_time";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
#[sbor(type_name = "ConsensusManagerCompareCurrentTimeInput")]
pub struct ConsensusManagerCompareCurrentTimeInputV1 {
    pub instant: Instant,
    pub precision: TimePrecisionV1,
    pub operator: TimeComparisonOperator,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
#[sbor(type_name = "ConsensusManagerCompareCurrentTimeInput")]
pub struct ConsensusManagerCompareCurrentTimeInputV2 {
    pub instant: Instant,
    pub precision: TimePrecisionV2,
    pub operator: TimeComparisonOperator,
}

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

pub type ConsensusManagerCreateValidatorOutput = (Global<ValidatorObjectTypeInfo>, Bucket, Bucket);

pub const VALIDATOR_REGISTER_IDENT: &str = "register";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorRegisterInput {}

pub type ValidatorRegisterOutput = ();

pub const VALIDATOR_UNREGISTER_IDENT: &str = "unregister";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUnregisterInput {}

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

pub type ValidatorUpdateKeyOutput = ();

pub const VALIDATOR_UPDATE_FEE_IDENT: &str = "update_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
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

pub const VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT: &str = "accepts_delegated_stake";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorAcceptsDelegatedStakeInput {}

pub type ValidatorAcceptsDelegatedStakeOutput = bool;

pub const VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT: &str = "total_stake_xrd_amount";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorTotalStakeXrdAmountInput {}

pub type ValidatorTotalStakeXrdAmountOutput = Decimal;

pub const VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT: &str = "total_stake_unit_supply";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorTotalStakeUnitSupplyInput {}

pub type ValidatorTotalStakeUnitSupplyOutput = Decimal;

pub const VALIDATOR_GET_REDEMPTION_VALUE_IDENT: &str = "get_redemption_value";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ValidatorGetRedemptionValueInput {
    pub amount_of_stake_units: Decimal,
}

pub type ValidatorGetRedemptionValueOutput = Decimal;

pub const VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS: &str = "signal_protocol_update_readiness";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorSignalProtocolUpdateReadinessInput {
    pub vote: String,
}

pub type ValidatorSignalProtocolUpdateReadinessOutput = ();

pub const VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT: &str = "get_protocol_update_readiness";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorGetProtocolUpdateReadinessInput {}

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

pub type ValidatorApplyEmissionOutput = ();

pub const VALIDATOR_APPLY_REWARD_IDENT: &str = "apply_reward";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorApplyRewardInput {
    /// A bucket with the rewarded XRDs (from transaction fees) for this validator.
    pub xrd_bucket: Bucket,
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

pub type ValidatorStartUnlockOwnerStakeUnitsOutput = ();

pub const VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT: &str = "finish_unlock_owner_stake_units";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ValidatorFinishUnlockOwnerStakeUnitsInput {}

pub type ValidatorFinishUnlockOwnerStakeUnitsOutput = Bucket;
