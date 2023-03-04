use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;

#[derive(ScryptoSbor)]
pub struct RegisterValidatorEvent;

#[derive(ScryptoSbor)]
pub struct UnregisterValidatorEvent;

#[derive(ScryptoSbor)]
pub struct StakeEvent {
    pub xrd_staked: Decimal,
}

#[derive(ScryptoSbor)]
pub struct UnstakeEvent {
    pub stake_units: Decimal, // TODO: Should be stake units instead?
}

#[derive(ScryptoSbor)]
pub struct ClaimXrdEvent {
    pub claimed_xrd: Decimal,
}

#[derive(ScryptoSbor)]
pub struct UpdateAcceptingStakeDelegationStateEvent {
    pub accepts_delegation: bool,
}
