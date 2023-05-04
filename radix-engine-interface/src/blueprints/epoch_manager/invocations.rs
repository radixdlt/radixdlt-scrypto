use crate::api::sorted_index_api::SortedKey;
use crate::blueprints::resource::*;
use crate::*;
use radix_engine_common::types::*;
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::math::Decimal;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

pub const EPOCH_MANAGER_BLUEPRINT: &str = "EpochManager";
pub const VALIDATOR_BLUEPRINT: &str = "Validator";

pub const EPOCH_MANAGER_CREATE_IDENT: &str = "create";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct EpochManagerCreateInput {
    pub validator_owner_token: [u8; NodeId::LENGTH], // TODO: Clean this up
    pub component_address: [u8; NodeId::LENGTH],     // TODO: Clean this up
    pub initial_epoch: u64,
    pub max_validators: u32,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
}

pub type EpochManagerCreateOutput = ();

pub const EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT: &str = "get_current_epoch";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct EpochManagerGetCurrentEpochInput;

pub type EpochManagerGetCurrentEpochOutput = u64;

pub const EPOCH_MANAGER_SET_EPOCH_IDENT: &str = "set_epoch";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct EpochManagerSetEpochInput {
    pub epoch: u64,
}

pub type EpochManagerSetEpochOutput = ();

pub const EPOCH_MANAGER_START_IDENT: &str = "start";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct EpochManagerStartInput {}

pub type EpochManagerStartOutput = ();

pub const EPOCH_MANAGER_NEXT_ROUND_IDENT: &str = "next_round";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct EpochManagerNextRoundInput {
    /// Current round number.
    /// Please note that in case of liveness breaks, this number may be different than previous
    /// reported `round + 1`. Such gaps are considered "round leader's fault" and are penalized
    /// on emission, according to leader reliability statistics (see `LeaderProposalMissHistory`).
    pub round: u64,

    /// A captured history of leader proposal misses since the previous reported round.
    pub leader_proposal_misses: LeaderProposalMissHistory,
}

impl EpochManagerNextRoundInput {
    /// Creates a "next round" input for a regular (happy-path, in terms of consensus) round
    /// progression, i.e. no missed proposals, no fallback rounds.
    /// Please note that the round number passed here should be an immediate successor of the
    /// previously reported round.
    pub fn successful(consecutive_round: u64) -> Self {
        Self {
            round: consecutive_round,
            leader_proposal_misses: LeaderProposalMissHistory {
                gap_round_leaders: Vec::new(),
                current_round_fallback_leader: None,
            },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct LeaderProposalMissHistory {
    /// The validators which were leaders of the "gap" rounds (i.e. those that were not reported to
    /// the epoch manager since the previous call; see `EpochManagerNextRoundInput::round`).
    /// This list will contain exactly `current_call.round - previous_call.round - 1` elements; in
    /// theory, this makes `EpochManagerNextRoundInput::round` field redundant (i.e. computable),
    /// but this relation can be used for an extra consistency check.
    /// The validators on this list should be penalized during emissions at the end of the current
    /// epoch.
    pub gap_round_leaders: Vec<ValidatorIndex>,

    /// An indication of the current round operating in a "fallback" mode (i.e. indicating a fault
    /// of the current leader).
    /// A `None` denotes a regular round (not a fallback).
    /// A `Some(current_leader)` specifies the current round's leader and indicates that current
    /// round is a fallback one (and that validator should be penalized during emissions, same as
    /// `gap_round_leaders`).
    /// Please note that this effect is independent from the gap rounds (i.e. the value of this
    /// field does not affect the invariants of `gap_round_leaders` or the round numbers).
    pub current_round_fallback_leader: Option<ValidatorIndex>,
}

/// An index of a specific validator within the current validator set.
/// To be exact: a `ValidatorIndex` equal to `k` references the `k-th` element returned by the
/// iterator of `BTreeMap<ComponentAddress, Validator>`.
/// This uniquely identifies the validator, while being shorter than `ComponentAddress` (we do care
/// about the constant factor of the space taken by `LeaderProposalMissHistory` under prolonged
/// liveness break scenarios).
pub type ValidatorIndex = usize;

pub type EpochManagerNextRoundOutput = ();

pub const EPOCH_MANAGER_CREATE_VALIDATOR_IDENT: &str = "create_validator";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct EpochManagerCreateValidatorInput {
    pub key: EcdsaSecp256k1PublicKey,
}

pub type EpochManagerCreateValidatorOutput = (ComponentAddress, Bucket);

pub const EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT: &str = "update_validator";

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
    pub lp_tokens: Bucket,
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

pub const VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT: &str = "update_accept_delegated_stake";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUpdateAcceptDelegatedStakeInput {
    pub accept_delegated_stake: bool,
}

pub type ValidatorUpdateAcceptDelegatedStakeOutput = ();
