use native_sdk::{LegacyDescribe, ScryptoSbor};
use radix_engine_interface::math::Decimal;

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct RegisterValidatorEvent;

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct UnregisterValidatorEvent;

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct StakeEvent {
    pub xrd_staked: Decimal,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct UnstakeEvent {
    pub stake_units: Decimal, // TODO: Should be stake units instead?
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct ClaimXrdEvent {
    pub claimed_xrd: Decimal,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct UpdateAcceptingStakeDelegationStateEvent {
    pub accepts_delegation: bool,
}
