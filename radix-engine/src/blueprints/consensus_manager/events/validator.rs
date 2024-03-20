use crate::internal_prelude::*;
use radix_common::math::Decimal;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct RegisterValidatorEvent;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct UnregisterValidatorEvent;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct StakeEvent {
    pub xrd_staked: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct UnstakeEvent {
    pub stake_units: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct ClaimXrdEvent {
    pub claimed_xrd: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct UpdateAcceptingStakeDelegationStateEvent {
    pub accepts_delegation: bool,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct ProtocolUpdateReadinessSignalEvent {
    pub protocol_version_name: String,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct ValidatorEmissionAppliedEvent {
    /// An epoch number of the *concluded* epoch (i.e. for which this emission applies).
    pub epoch: Epoch,
    /// An amount of XRD in the validator's stake pool, captured *before* this emission.
    pub starting_stake_pool_xrd: Decimal,
    /// An amount of XRD added to the validator's stake pool from this epoch's emissions.
    /// Note: this number represents the net emission, after any applicable reliability penalty
    /// and validator fee have been subtracted.
    pub stake_pool_added_xrd: Decimal,
    /// A total supply of stake units of the validator at the moment of applying this emission.
    /// Note: calculating `stake_pool_added_xrd / total_stake_unit_supply` gives a convenient "XRD
    /// emitted per stake unit" factor, which may be used to easily calculate individual staker's
    /// gains.
    /// Note: this number is captured *before* auto-staking of the validator fee described below.
    pub total_stake_unit_supply: Decimal,
    /// An amount of XRD received by the validator's owner (according to the configured fee
    /// percentage).
    /// Note: this fee is automatically staked and placed inside the owner's stake vault (internal
    /// to the validator).
    /// Note: calculating `stake_pool_added_xrd + validator_fee_xrd` gives the total emission for
    /// this validator (entirety of which goes into its stake pool XRD vault).
    /// Note: calculating `validator_fee_xrd / (stake_pool_added_xrd + validator_fee_xrd)` gives the
    /// validator's configured fee percentage effective during the emission period.
    pub validator_fee_xrd: Decimal,
    /// A number of proposals successfully made by this validator during the emission period.
    pub proposals_made: u64,
    /// A number of proposals missed by this validator during the emission period.
    pub proposals_missed: u64,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct ValidatorRewardAppliedEvent {
    /// An epoch number of the *concluded* epoch (i.e. for which this reward applies).
    pub epoch: Epoch,
    /// The reward amount
    pub amount: Decimal,
}
