use crate::types::*;
use radix_engine_interface::math::Decimal;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct RegisterValidatorEvent;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct UnregisterValidatorEvent;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct StakeEvent {
    pub xrd_staked: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct UnstakeEvent {
    pub stake_units: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct ClaimXrdEvent {
    pub claimed_xrd: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct UpdateAcceptingStakeDelegationStateEvent {
    pub accepts_delegation: bool,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct RewardAppliedEvent {
    /// An amount of XRD added to the validator's stake as a reward.
    /// Note: this number represent a final gain of value of all stake units of the validator, which
    /// means that any applicable reliability penalty and validator fee were already subtracted.
    pub stake_added_xrd: Decimal,
    /// A total supply of stake units of the validator at the moment of applying this reward.
    /// Note: calculating `stake_added_xrd / total_su_supply` gives a convenient "XRD reward per
    /// stake unit" factor, which may be used to easily calculate individual staker's gains.
    /// Note: this number is captured *before* auto-staking of the validator fee described below.
    pub total_su_supply: Decimal,
    /// An amount of XRD received by the validator's owner (according to the configured fee
    /// percentage).
    /// Note: this fee is automatically staked and placed inside the owner's stake vault (internal
    /// to the validator).
    /// Note: calculating `validator_fee_xrd / (stake_added_xrd + validator_fee_xrd)` gives the
    /// validator's configured fee percentage effective during the rewarded period.
    pub validator_fee_xrd: Decimal,
    /// A number of proposals successfully made by this validator during the rewarded period.
    pub proposals_made: u64,
    /// A number of proposals missed by this validator during the rewarded period.
    pub proposals_missed: u64,
}
