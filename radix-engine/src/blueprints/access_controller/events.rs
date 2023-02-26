use native_sdk::{LegacyDescribe, ScryptoSbor};
use radix_engine_interface::blueprints::access_controller::{Proposer, RecoveryProposal};

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct InitiateRecoveryEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct RuleSetUpdateEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct CancelRecoveryProposalEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct LockPrimaryRoleEvent;

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct UnlockPrimaryRoleEvent;

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct StopTimedRecoveryEvent;
