use radix_engine_interface::blueprints::access_controller::{Proposer, RecoveryProposal};
use radix_engine_interface::*;

#[derive(ScryptoSbor)]
pub struct InitiateRecoveryEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor)]
pub struct RuleSetUpdateEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor)]
pub struct CancelRecoveryProposalEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor)]
pub struct LockPrimaryRoleEvent;

#[derive(ScryptoSbor)]
pub struct UnlockPrimaryRoleEvent;

#[derive(ScryptoSbor)]
pub struct StopTimedRecoveryEvent;
