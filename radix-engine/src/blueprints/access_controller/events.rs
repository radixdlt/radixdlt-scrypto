use crate::types::*;
use radix_engine_interface::blueprints::access_controller::{Proposer, RecoveryProposal};

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct InitiateRecoveryEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct InitiateBadgeWithdrawAttemptEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct RuleSetUpdateEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct BadgeWithdrawEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct CancelRecoveryProposalEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct CancelBadgeWithdrawAttemptEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct LockPrimaryRoleEvent;

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct UnlockPrimaryRoleEvent;

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct StopTimedRecoveryEvent;
