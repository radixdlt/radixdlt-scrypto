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
