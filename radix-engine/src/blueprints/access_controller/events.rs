use crate::types::*;
use radix_engine_interface::blueprints::access_controller::{Proposer, RecoveryProposal};

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct InitiateRecoveryEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct RuleSetUpdateEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct CancelRecoveryProposalEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct LockPrimaryRoleEvent;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct UnlockPrimaryRoleEvent;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct StopTimedRecoveryEvent;
